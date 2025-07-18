use crate::user::UserService;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MockUserService {
    users: HashMap<String, String>,
}

impl MockUserService {
    pub fn new(users: HashMap<String, String>) -> Self {
        Self { users }
    }
}

#[async_trait]
impl UserService for MockUserService {
    async fn get_cid(&self, access_token: &str) -> anyhow::Result<String> {
        self.users
            .get(access_token)
            .ok_or(anyhow::anyhow!("invalid access token"))
            .cloned()
    }
}
