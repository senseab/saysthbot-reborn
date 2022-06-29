use migration::{Migrator, MigratorTrait};
use models::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait,
};
use wd_log::{log_error_ln, log_info_ln, log_panic, log_warn_ln};

const PAGE_SIZE: usize = 25;

#[derive(Debug)]
pub struct Controller {
    db: DatabaseConnection,
}

pub struct PaginatedRecordData {
    pub items_count: usize,
    pub pages_count: usize,
    pub current_data: Vec<(RecordModel, Option<UserModel>)>,
}

impl Controller {
    /// Create controller
    pub async fn new(config: String) -> Result<Self, DbErr> {
        Ok(Self {
            db: Database::connect(config).await?,
        })
    }

    /// Do migrate
    pub async fn migrate(&self) -> Result<(), DbErr> {
        if let Err(err) = Migrator::install(&self.db).await {
            log_warn_ln!("{}", err)
        }
        if let Err(err) = Migrator::up(&self.db, None).await {
            Err(err)
        } else {
            log_info_ln!("database initialized.");
            Ok(())
        }
    }

    /// register user when `/start` command called.
    pub async fn register_user(&self, user_id: &i64, username: &String) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        self.setup_user(user_id, username, &transaction).await?;
        transaction.commit().await
    }

    /// update user notify when `/mute` or `/unmute` command called.
    pub async fn set_user_notify(&self, user_id: &i64, notify: bool) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        if let Some(user) = self.get_user(user_id, &transaction).await? {
            let mut user_active: UserActiveModel = user.into();
            user_active.notify = Set(notify);
            user_active.save(&transaction).await?;
        }
        transaction.commit().await
    }

    pub async fn get_user_notify(&self, user_id: &i64) -> Result<bool, DbErr> {
        let transaction = self.db.begin().await?;
        if let Some(user) = self.get_user(&user_id, &transaction).await? {
            Ok(user.notify)
        } else {
            Ok(false)
        }
    }

    async fn setup_user(
        &self,
        user_id: &i64,
        username: &String,
        transaction: &DatabaseTransaction,
    ) -> Result<UserActiveModel, DbErr> {
        match self.get_user(user_id, &transaction).await? {
            Some(user) => {
                let mut user_active: UserActiveModel = user.into();
                user_active.username = Set(Some(username.to_string()));
                user_active.save(transaction).await
            }
            None => {
                UserActiveModel {
                    tg_uid: Set(user_id.to_owned()),
                    username: Set(Some(username.to_string())),
                    notify: Set(true),
                    ..Default::default()
                }
                .save(transaction)
                .await
            }
        }
    }

    async fn get_user(
        &self,
        user_id: &i64,
        transaction: &DatabaseTransaction,
    ) -> Result<Option<UserModel>, DbErr> {
        User::find()
            .filter(UserColumn::Id.eq(user_id.to_owned()))
            .one(transaction)
            .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserModel>, DbErr> {
        let transaction = self.db.begin().await?;
        User::find()
            .filter(UserColumn::Username.eq(username.to_owned()))
            .one(&transaction)
            .await
    }

    /// get records when inline query called.
    pub async fn get_records_by_keywords(
        &self,
        key_word: &String,
    ) -> Result<PaginatedRecordData, DbErr> {
        let pagination = Record::find()
            .find_also_related(User)
            .filter(RecordColumn::Message.contains(key_word.as_str()))
            .paginate(&self.db, PAGE_SIZE * 2); // 50 records seems ok.
        Ok(PaginatedRecordData {
            items_count: pagination.num_items().await?,
            pages_count: pagination.num_pages().await?,
            current_data: pagination.fetch().await?,
        })
    }

    /// get records when `/list` command called or inline button request.
    pub async fn get_records_by_userid_with_pagination(
        &self,
        user_id: i64,
        page: usize,
    ) -> Result<Option<PaginatedRecordData>, DbErr> {
        let transaction = self.db.begin().await?;
        if let Some(user) = self.get_user(&user_id, &transaction).await? {
            let pagination = Record::find()
                .find_also_related(User)
                .filter(RecordColumn::UserId.eq(user.id))
                .paginate(&transaction, PAGE_SIZE);
            Ok(Some(PaginatedRecordData {
                current_data: pagination.fetch_page(page).await?,
                items_count: pagination.num_items().await?,
                pages_count: pagination.num_pages().await?,
            }))
        } else {
            log_error_ln!("cannot find user tg_uid={}", user_id);
            Ok(None)
        }
    }

    /// add record forward a message to bot.
    pub async fn add_record(
        &self,
        user_id: i64,
        username: &String,
        text: String,
    ) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        let user = self.setup_user(&user_id, &username, &transaction).await?;
        RecordActiveModel {
            message: Set(text),
            user_id: user.id,
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
        transaction.commit().await
    }

    /// del record when `/delete` command called.
    pub async fn del_record(&self, id: i64, user_id: i64) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        if let Some(user) = self.get_user(&user_id, &transaction).await? {
            RecordActiveModel {
                id: Set(id),
                user_id: Set(user.id),
                ..Default::default()
            }
            .delete(&transaction)
            .await?;
        }
        transaction.commit().await
    }

    pub fn err_handler(&self, error: DbErr) {
        match error {
            DbErr::Conn(err) => log_panic!("{}", err),
            error => log_error_ln!("{}", error),
        }
    }
}
