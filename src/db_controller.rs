use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, DbErr};
use wd_log::{log_error_ln, log_info_ln};

#[derive(Debug)]
pub struct Controller {
    db: DatabaseConnection,
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

    fn db_error_handler(error: DbErr) {
        log_error_ln!("{}", error)
    }
}
