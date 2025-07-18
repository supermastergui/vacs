use crate::user::UserService;
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;
use tracing::instrument;

pub struct ConnectUserService {
    user_details_endpoint_url: String,
    client: reqwest::Client,
}

impl ConnectUserService {
    pub fn new(user_info_endpoint_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            user_details_endpoint_url: user_info_endpoint_url.to_owned(),
            client: reqwest::ClientBuilder::new()
                .user_agent(crate::APP_USER_AGENT)
                .build()
                .context("Failed to create HTTP client")?,
        })
    }
}

#[async_trait]
impl UserService for ConnectUserService {
    #[instrument(level = "debug", skip_all, err)]
    async fn get_cid(&self, access_token: &str) -> anyhow::Result<String> {
        tracing::trace!("Performing HTTP request");
        let response = self
            .client
            .get(&self.user_details_endpoint_url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to perform HTTP request")?
            .error_for_status()
            .context("Received non-200 HTTP status code")?;

        tracing::trace!(content_length = ?response.content_length(), "Parsing response body");
        let user_details = response
            .json::<ConnectUserDetails>()
            .await
            .context("Failed to parse response body")?;

        tracing::debug!(?user_details, "Successfully retrieved user details");
        Ok(user_details.data.cid)
    }
}

#[derive(Deserialize, Debug)]
struct ConnectUserDetails {
    data: ConnectUserDetailsData,
}

#[derive(Deserialize, Debug)]
struct ConnectUserDetailsData {
    cid: String,
}
