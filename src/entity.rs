use chrono::{DateTime, Utc};

#[crud_enable]
#[derive(Clone, Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password: String, // sha256
    pub group_list: Vec<u64>,
    pub online: bool,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct UserMsg {
    pub id: u64,
    pub from_id: u64,
    pub to_id: u64,
    pub date_time: DateTime<Utc>,
    pub content: String,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct Group {
    pub id: u64,
    pub group_name: String,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct GroupMember {
    pub id: u64,
    pub group_id: u64,
    pub user_id: u64,
    pub role_id: u64,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct GroupMsg {
    pub id: u64,
    pub group_id: u64,
    pub user_id: u64,
    pub date_time: DateTime<Utc>,
    pub content: String,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct Role {
    pub id: u64,
    pub role_name: String,
}
