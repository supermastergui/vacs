use crate::oauth::OAuthClient;
use anyhow::Context;
use async_trait::async_trait;
use oauth2::basic::BasicClient;
use oauth2::url::Url;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, TokenResponse,
    TokenUrl,
};
use tracing::instrument;

pub struct OAuthConfig {
    auth_url: String,
    token_url: String,
    redirect_url: String,
    client_id: String,
    client_secret: String,
}

pub struct ConnectOAuthClient {
    client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    http_client: reqwest::Client,
}

impl ConnectOAuthClient {
    pub fn new(config: OAuthConfig) -> anyhow::Result<Self> {
        let client = BasicClient::new(ClientId::new(config.client_id))
            .set_client_secret(ClientSecret::new(config.client_secret))
            .set_auth_uri(AuthUrl::new(config.auth_url).context("Invalid auth URL")?)
            .set_token_uri(TokenUrl::new(config.token_url).context("Invalid token URL")?)
            .set_redirect_uri(
                RedirectUrl::new(config.redirect_url).context("Invalid redirect URL")?,
            );

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(crate::APP_USER_AGENT)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            http_client,
        })
    }
}

#[async_trait]
impl OAuthClient for ConnectOAuthClient {
    #[instrument(level = "debug", skip_all)]
    fn auth_url(&self) -> (Url, CsrfToken, PkceCodeVerifier) {
        tracing::trace!("Generating VATSIM OAuth2 URL");

        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(challenge)
            .url();
        (url, csrf_token, verifier)
    }

    #[instrument(level = "debug", skip_all, err)]
    async fn exchange_code(
        &self,
        code: AuthorizationCode,
        verifier: PkceCodeVerifier,
    ) -> anyhow::Result<(AccessToken, RefreshToken)> {
        tracing::trace!("Exchanging OAuth2 code for token");

        let response = self
            .client
            .exchange_code(code)
            .set_pkce_verifier(verifier)
            .request_async(&self.http_client)
            .await
            .context("Failed to exchange OAuth2 code for token")?;

        if response.refresh_token().is_none() {
            tracing::warn!("No refresh token received");
            anyhow::bail!("No refresh token received");
        }

        Ok((
            response.access_token().clone(),
            response.refresh_token().unwrap().clone(),
        ))
    }

    #[instrument(level = "debug", skip_all, err)]
    async fn refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> anyhow::Result<(AccessToken, RefreshToken)> {
        tracing::trace!("Refreshing OAuth2 token");
        let response = self
            .client
            .exchange_refresh_token(refresh_token)
            .request_async(&self.http_client)
            .await
            .context("Failed to refresh OAuth2 token")?;

        if response.refresh_token().is_none() {
            tracing::warn!("No refresh token received");
            anyhow::bail!("No refresh token received");
        }

        Ok((
            response.access_token().clone(),
            response.refresh_token().unwrap().clone(),
        ))
    }
}
