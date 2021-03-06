use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "records")]
pub struct Model {
    /// internal ID
    #[sea_orm(primary_key)]
    pub id: i64,

    /// relation user id
    #[sea_orm(indexed)]
    pub user_id: i64,

    /// records
    #[sea_orm(indexed, column_type = "Text", unique)]
    pub message: String,

    /// hot
    #[sea_orm(default_value = "0")]
    pub hot: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
