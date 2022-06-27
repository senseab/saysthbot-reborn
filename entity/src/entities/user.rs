use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// internal ID
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Telegram user ID
    #[sea_orm(unique)]
    pub tg_uid: i64,

    /// Telegram user name
    #[sea_orm(nullable)]
    pub username: Option<String>,

    /// use notify
    #[sea_orm(default_value = true)]
    pub notify: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::record::Entity")]
    Record,
}

impl Related<super::record::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Record.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
