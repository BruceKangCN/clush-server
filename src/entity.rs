use chrono::{DateTime, Utc};

#[crud_enable]
#[derive(Clone, Debug)]
pub struct User {
    id: u64,
    username: String,
    password: String, // sha256
    group_list: Vec<u64>,
    online: bool,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct UserMsg {
    id: u64,
    from_id: u64,
    to_id: u64,
    date_time: DateTime<Utc>,
    content: String,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct Group {
    id: u64,
    group_name: String,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct GroupMember {
    id: u64,
    group_id: u64,
    user_id: u64,
    role_id: u64,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct GroupMsg {
    id: u64,
    group_id: u64,
    user_id: u64,
    date_time: DateTime<Utc>,
    content: String,
}

#[crud_enable]
#[derive(Clone, Debug)]
pub struct Role {
    id: u64,
    role_name: String,
}
