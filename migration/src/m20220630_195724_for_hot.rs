use models::prelude::Record;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220630_195724_for_hot"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Record)
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("hot")).big_integer().default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Record)
                    .drop_column(Alias::new("hot"))
                    .to_owned(),
            )
            .await
    }
}
