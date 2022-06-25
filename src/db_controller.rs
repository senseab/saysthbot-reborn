use migration::{Migrator, MigratorTrait};
use models::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait,
};
use wd_log::{log_error_ln, log_info_ln};

const page_size: usize = 25;

#[derive(Debug)]
pub struct Controller {
    db: DatabaseConnection,
}

pub struct PaginatedRecordData {
    pub items_count: usize,
    pub pages_count: usize,
    pub current_data: Vec<RecordModel>,
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
            log_info_ln!("{}", err)
        }
        Migrator::up(&self.db, None).await
    }

    /// register user when `/start` command called.
    pub async fn register_user(&self, user_id: u64, username: String) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        self.setup_user(user_id, username, &transaction).await?;
        transaction.commit().await
    }

    /// update user notify when `/mute` or `/unmute` command called.
    pub async fn set_user_notify(&self, user_id: u64, notify: bool) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        if let Some(user) = self.get_user(user_id, &transaction).await? {
            let mut user_active: UserActiveModel = user.into();
            user_active.notify = Set(notify);
            user_active.save(&transaction).await?;
        }
        transaction.commit().await
    }

    async fn setup_user(
        &self,
        user_id: u64,
        username: String,
        transaction: &DatabaseTransaction,
    ) -> Result<UserActiveModel, DbErr> {
        match self.get_user(user_id, &transaction).await? {
            Some(user) => {
                let mut user_active: UserActiveModel = user.into();
                user_active.username = Set(Some(username));
                user_active.save(transaction).await
            }
            None => {
                UserActiveModel {
                    tg_uid: Set(user_id),
                    username: Set(Some(username)),
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
        user_id: u64,
        transaction: &DatabaseTransaction,
    ) -> Result<Option<UserModel>, DbErr> {
        User::find()
            .filter(UserColumn::TgUid.eq(user_id))
            .one(transaction)
            .await
    }

    /// get records when inline query called.
    pub async fn get_records_by_keywords(
        &self,
        key_word: &String,
    ) -> Result<PaginatedRecordData, DbErr> {
        let pagination = Record::find()
            .filter(RecordColumn::Message.contains(key_word.as_str()))
            .paginate(&self.db, page_size * 2); // 50 records seems ok.
        Ok(PaginatedRecordData {
            items_count: pagination.num_items().await?,
            pages_count: pagination.num_pages().await?,
            current_data: pagination.fetch().await?,
        })
    }

    /// get records when `/list` command called or inline button request.
    pub async fn get_records_by_userid_with_pagination(
        &self,
        user_id: u64,
        page: usize,
    ) -> Result<Option<PaginatedRecordData>, DbErr> {
        let transaction = self.db.begin().await?;
        if let Some(user) = self.get_user(user_id, &transaction).await? {
            let pagination = Record::find()
                .filter(RecordColumn::UserId.eq(user.id))
                .paginate(&transaction, page_size);
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
        user_id: u64,
        username: String,
        text: String,
    ) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        let user = self.setup_user(user_id, username, &transaction).await?;
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
    pub async fn del_record(&self, id: u64) -> Result<(), DbErr> {
        let transaction = self.db.begin().await?;
        RecordActiveModel {
            id: Set(id),
            ..Default::default()
        }
        .delete(&transaction)
        .await?;
        transaction.commit().await
    }
}
