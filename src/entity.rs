use chrono::{DateTime, Utc};

#[crud_enable(table_name: public."user")]
#[derive(Clone, Debug)]
pub struct User {
    pub id: Option<u64>,
    pub username: Option<String>,
    pub password: Option<String>, // sha256
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct UserMsg {
    pub id: Option<u64>,
    pub from_id: Option<u64>,
    pub to_id: Option<u64>,
    pub date_time: Option<DateTime<Utc>>,
    pub content: Option<String>,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct Group {
    pub id: Option<u64>,
    pub group_name: Option<String>,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct GroupMember {
    pub id: Option<u64>,
    pub group_id: Option<u64>,
    pub user_id: Option<u64>,
    pub role_id: Option<u64>,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct GroupMsg {
    pub id: Option<u64>,
    pub group_id: Option<u64>,
    pub user_id: Option<u64>,
    pub date_time: Option<DateTime<Utc>>,
    pub content: Option<String>,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct Role {
    pub id: Option<u64>,
    pub role_name: Option<String>,
}
