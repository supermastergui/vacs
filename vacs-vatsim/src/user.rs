pub mod connect;
pub mod mock;

use async_trait::async_trait;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_cid(&self, access_token: &str) -> anyhow::Result<String>;
}
