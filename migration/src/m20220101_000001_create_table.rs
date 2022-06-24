use models::*;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Schema},
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let builder = db.get_database_backend();
        let schema = Schema::new(builder);

        db.execute(builder.build(&schema.create_table_from_entity(user::Entity)))
            .await?;

        db.execute(builder.build(&schema.create_table_from_entity(record::Entity)))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(user::Entity).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(record::Entity).to_owned())
            .await?;

        Ok(())
    }
}
