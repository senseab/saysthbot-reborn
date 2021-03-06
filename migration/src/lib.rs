pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20220625_222908_message_unique;
mod m20220630_195724_for_hot;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20220625_222908_message_unique::Migration),
            Box::new(m20220630_195724_for_hot::Migration),
        ]
    }
}
