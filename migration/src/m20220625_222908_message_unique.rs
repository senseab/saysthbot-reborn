use models::prelude::{Record, RecordColumn};
use sea_orm_migration::prelude::*;

pub struct Migration;

const RECORD_MESSAGE_UNIQUE: &str = "record_message_unique";

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220625_222908_message_unique"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(Record)
                    .col(RecordColumn::Message)
                    .name(RECORD_MESSAGE_UNIQUE)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(Record)
                    .name(RECORD_MESSAGE_UNIQUE)
                    .to_owned(),
            )
            .await
    }
}
