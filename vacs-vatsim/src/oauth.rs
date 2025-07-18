pub mod connect;
pub mod mock;

use async_trait::async_trait;
use oauth2::{url::Url, AccessToken, AuthorizationCode, CsrfToken, PkceCodeVerifier, RefreshToken};

#[async_trait]
pub trait OAuthClient: Send + Sync {
    fn auth_url(&self) -> (Url, CsrfToken, PkceCodeVerifier);
    async fn exchange_code(
        &self,
        code: AuthorizationCode,
        verifier: PkceCodeVerifier,
    ) -> anyhow::Result<(AccessToken, RefreshToken)>;
    async fn refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> anyhow::Result<(AccessToken, RefreshToken)>;
}
