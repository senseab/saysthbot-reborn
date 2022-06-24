use rbatis::crud_table;

#[crud_table]
#[derive(Debug, Clone)]
/// User
pub struct User {
    /// Internal id
    pub id: Option<u64>,

    /// Telegram user id
    pub tg_id: Option<u64>,

    /// Telegram user name
    pub username: Option<String>,

    /// notify enabled
    pub notify: Option<bool>,
}

#[crud_table]
#[derive(Debug, Clone)]
/// Record
pub struct Record {
    /// Internal id
    pub id: Option<u64>,

    /// User ID
    pub user_id: Option<u64>,

    /// Record
    pub record: Option<String>,
}
