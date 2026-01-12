# Spec 377: OAuth2 Support

## Phase
17 - Authentication/Authorization

## Spec ID
377

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~12%

---

## Objective

Implement OAuth2 authentication support for third-party identity providers (Google, GitHub, Microsoft, etc.). This enables users to authenticate using their existing accounts from trusted providers, reducing friction and improving security by leveraging established identity systems.

---

## Acceptance Criteria

- [ ] Implement OAuth2 authorization code flow
- [ ] Support multiple OAuth2 providers
- [ ] Handle OAuth2 state parameter for CSRF protection
- [ ] Implement token exchange
- [ ] Fetch and normalize user info from providers
- [ ] Support auto-creation of users on first OAuth login
- [ ] Link OAuth identities to existing accounts
- [ ] Handle token refresh for providers
- [ ] Support OpenID Connect (OIDC) where available

---

## Implementation Details

### OAuth2 Provider Implementation

```rust
// src/auth/oauth.rs

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};
use url::Url;

use crate::auth::{
    config::OAuth2ProviderConfig,
    events::{AuthEvent, AuthEventEmitter},
    provider::{AuthProvider, User, UserRepository},
    types::*,
};

/// OAuth2 authorization URL response
#[derive(Debug, Clone, Serialize)]
pub struct AuthorizationUrl {
    pub url: String,
    pub state: String,
}

/// OAuth2 token response from provider
#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    /// OpenID Connect ID token
    pub id_token: Option<String>,
}

/// Normalized user info from OAuth2 provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2UserInfo {
    /// Provider-specific user ID
    pub provider_id: String,
    /// Provider name
    pub provider: String,
    /// Email address
    pub email: Option<String>,
    /// Whether email is verified
    pub email_verified: bool,
    /// Display name
    pub name: Option<String>,
    /// Profile picture URL
    pub picture: Option<String>,
    /// Username/login
    pub username: Option<String>,
    /// Raw provider response
    pub raw: serde_json::Value,
}

/// OAuth2 state for CSRF protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2State {
    /// Random state value
    pub state: String,
    /// Provider name
    pub provider: String,
    /// Redirect URI after successful auth
    pub redirect_uri: Option<String>,
    /// When the state was created
    pub created_at: DateTime<Utc>,
    /// Whether this is a link operation (linking to existing account)
    pub link_user_id: Option<UserId>,
}

impl OAuth2State {
    pub fn new(provider: &str) -> Self {
        Self {
            state: generate_state(),
            provider: provider.to_string(),
            redirect_uri: None,
            created_at: Utc::now(),
            link_user_id: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.created_at + Duration::minutes(10)
    }
}

/// Generate random state for CSRF protection
fn generate_state() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

/// Linked OAuth identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedIdentity {
    pub id: String,
    pub user_id: UserId,
    pub provider: String,
    pub provider_id: String,
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// OAuth2 provider trait
#[async_trait]
pub trait OAuth2ProviderTrait: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Build authorization URL
    fn authorization_url(&self, state: &str) -> String;

    /// Exchange code for tokens
    async fn exchange_code(&self, code: &str) -> AuthResult<OAuth2TokenResponse>;

    /// Get user info from provider
    async fn get_user_info(&self, access_token: &str) -> AuthResult<OAuth2UserInfo>;

    /// Refresh access token (if supported)
    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<OAuth2TokenResponse>;
}

/// Generic OAuth2 provider implementation
pub struct GenericOAuth2Provider {
    config: OAuth2ProviderConfig,
    http_client: Client,
}

impl GenericOAuth2Provider {
    pub fn new(config: OAuth2ProviderConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl OAuth2ProviderTrait for GenericOAuth2Provider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn authorization_url(&self, state: &str) -> String {
        let mut url = Url::parse(&self.config.auth_url).unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("state", state)
            .append_pair("scope", &self.config.scopes.join(" "));

        url.to_string()
    }

    async fn exchange_code(&self, code: &str) -> AuthResult<OAuth2TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Token exchange failed: {}", e)))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AuthError::OAuth2Error(format!("Token exchange failed: {}", error)));
        }

        response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse token response: {}", e)))
    }

    async fn get_user_info(&self, access_token: &str) -> AuthResult<OAuth2UserInfo> {
        let userinfo_url = self.config.userinfo_url.as_ref().ok_or_else(|| {
            AuthError::OAuth2Error("User info URL not configured".to_string())
        })?;

        let response = self
            .http_client
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("User info request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthError::OAuth2Error("Failed to get user info".to_string()));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse user info: {}", e)))?;

        // Normalize user info based on common fields
        Ok(OAuth2UserInfo {
            provider_id: raw
                .get("id")
                .or_else(|| raw.get("sub"))
                .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|n| n.to_string()).as_deref().map(|s| s)))
                .map(String::from)
                .unwrap_or_default(),
            provider: self.name().to_string(),
            email: raw.get("email").and_then(|v| v.as_str()).map(String::from),
            email_verified: raw
                .get("email_verified")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            name: raw
                .get("name")
                .or_else(|| raw.get("displayName"))
                .and_then(|v| v.as_str())
                .map(String::from),
            picture: raw
                .get("picture")
                .or_else(|| raw.get("avatar_url"))
                .and_then(|v| v.as_str())
                .map(String::from),
            username: raw
                .get("login")
                .or_else(|| raw.get("username"))
                .and_then(|v| v.as_str())
                .map(String::from),
            raw,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<OAuth2TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Token refresh failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthError::OAuth2Error("Token refresh failed".to_string()));
        }

        response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse refresh response: {}", e)))
    }
}

/// GitHub-specific OAuth2 provider
pub struct GitHubOAuth2Provider {
    config: OAuth2ProviderConfig,
    http_client: Client,
}

impl GitHubOAuth2Provider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let config = OAuth2ProviderConfig {
            name: "github".to_string(),
            client_id,
            client_secret,
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            userinfo_url: Some("https://api.github.com/user".to_string()),
            scopes: vec!["user:email".to_string()],
            redirect_uri,
            auto_create_users: true,
            default_roles: vec!["user".to_string()],
        };

        Self {
            config,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl OAuth2ProviderTrait for GitHubOAuth2Provider {
    fn name(&self) -> &str {
        "github"
    }

    fn authorization_url(&self, state: &str) -> String {
        format!(
            "{}?client_id={}&redirect_uri={}&scope={}&state={}",
            self.config.auth_url,
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            self.config.scopes.join(" "),
            state
        )
    }

    async fn exchange_code(&self, code: &str) -> AuthResult<OAuth2TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.config.redirect_uri.as_str()),
        ];

        let response = self
            .http_client
            .post(&self.config.token_url)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("GitHub token exchange failed: {}", e)))?;

        response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse GitHub token: {}", e)))
    }

    async fn get_user_info(&self, access_token: &str) -> AuthResult<OAuth2UserInfo> {
        // Get user info
        let user_response = self
            .http_client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("User-Agent", "Tachikoma")
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("GitHub user request failed: {}", e)))?;

        let user: serde_json::Value = user_response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse GitHub user: {}", e)))?;

        // Get emails
        let emails_response = self
            .http_client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "Tachikoma")
            .send()
            .await
            .ok();

        let primary_email = if let Some(response) = emails_response {
            let emails: Vec<serde_json::Value> = response.json().await.unwrap_or_default();
            emails
                .iter()
                .find(|e| e.get("primary").and_then(|v| v.as_bool()).unwrap_or(false))
                .and_then(|e| e.get("email").and_then(|v| v.as_str()))
                .map(String::from)
        } else {
            user.get("email").and_then(|v| v.as_str()).map(String::from)
        };

        Ok(OAuth2UserInfo {
            provider_id: user.get("id").and_then(|v| v.as_i64()).map(|n| n.to_string()).unwrap_or_default(),
            provider: "github".to_string(),
            email: primary_email,
            email_verified: true, // GitHub verifies emails
            name: user.get("name").and_then(|v| v.as_str()).map(String::from),
            picture: user.get("avatar_url").and_then(|v| v.as_str()).map(String::from),
            username: user.get("login").and_then(|v| v.as_str()).map(String::from),
            raw: user,
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> AuthResult<OAuth2TokenResponse> {
        Err(AuthError::OAuth2Error("GitHub doesn't support token refresh".to_string()))
    }
}

/// Google OAuth2 provider
pub struct GoogleOAuth2Provider {
    config: OAuth2ProviderConfig,
    http_client: Client,
}

impl GoogleOAuth2Provider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let config = OAuth2ProviderConfig {
            name: "google".to_string(),
            client_id,
            client_secret,
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            redirect_uri,
            auto_create_users: true,
            default_roles: vec!["user".to_string()],
        };

        Self {
            config,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl OAuth2ProviderTrait for GoogleOAuth2Provider {
    fn name(&self) -> &str {
        "google"
    }

    fn authorization_url(&self, state: &str) -> String {
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
            self.config.auth_url,
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&self.config.scopes.join(" ")),
            state
        )
    }

    async fn exchange_code(&self, code: &str) -> AuthResult<OAuth2TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Google token exchange failed: {}", e)))?;

        response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse Google token: {}", e)))
    }

    async fn get_user_info(&self, access_token: &str) -> AuthResult<OAuth2UserInfo> {
        let response = self
            .http_client
            .get(self.config.userinfo_url.as_ref().unwrap())
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Google user info failed: {}", e)))?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse Google user: {}", e)))?;

        Ok(OAuth2UserInfo {
            provider_id: raw.get("sub").and_then(|v| v.as_str()).map(String::from).unwrap_or_default(),
            provider: "google".to_string(),
            email: raw.get("email").and_then(|v| v.as_str()).map(String::from),
            email_verified: raw.get("email_verified").and_then(|v| v.as_bool()).unwrap_or(false),
            name: raw.get("name").and_then(|v| v.as_str()).map(String::from),
            picture: raw.get("picture").and_then(|v| v.as_str()).map(String::from),
            username: raw.get("email").and_then(|v| v.as_str()).map(|e| e.split('@').next().unwrap_or("").to_string()),
            raw,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<OAuth2TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Google refresh failed: {}", e)))?;

        response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2Error(format!("Failed to parse Google refresh: {}", e)))
    }
}

/// OAuth2 manager for handling OAuth2 flows
pub struct OAuth2Manager {
    providers: HashMap<String, Arc<dyn OAuth2ProviderTrait>>,
    state_storage: Arc<dyn OAuth2StateStorage>,
    identity_storage: Arc<dyn LinkedIdentityStorage>,
    user_repository: Arc<dyn UserRepository>,
    event_emitter: Arc<dyn AuthEventEmitter>,
}

impl OAuth2Manager {
    pub fn new(
        state_storage: Arc<dyn OAuth2StateStorage>,
        identity_storage: Arc<dyn LinkedIdentityStorage>,
        user_repository: Arc<dyn UserRepository>,
        event_emitter: Arc<dyn AuthEventEmitter>,
    ) -> Self {
        Self {
            providers: HashMap::new(),
            state_storage,
            identity_storage,
            user_repository,
            event_emitter,
        }
    }

    /// Register a provider
    pub fn register_provider(&mut self, provider: Arc<dyn OAuth2ProviderTrait>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    /// Start OAuth2 flow
    #[instrument(skip(self), fields(provider = %provider_name))]
    pub async fn start_auth(
        &self,
        provider_name: &str,
        link_user_id: Option<UserId>,
    ) -> AuthResult<AuthorizationUrl> {
        let provider = self.get_provider(provider_name)?;

        let mut state = OAuth2State::new(provider_name);
        state.link_user_id = link_user_id;

        self.state_storage.store(&state).await?;

        let url = provider.authorization_url(&state.state);

        Ok(AuthorizationUrl {
            url,
            state: state.state,
        })
    }

    /// Handle OAuth2 callback
    #[instrument(skip(self, code), fields(state = %state))]
    pub async fn handle_callback(
        &self,
        provider_name: &str,
        code: &str,
        state: &str,
    ) -> AuthResult<AuthIdentity> {
        // Verify state
        let stored_state = self
            .state_storage
            .get(state)
            .await?
            .ok_or(AuthError::OAuth2Error("Invalid state".to_string()))?;

        if stored_state.is_expired() {
            return Err(AuthError::OAuth2Error("State expired".to_string()));
        }

        if stored_state.provider != provider_name {
            return Err(AuthError::OAuth2Error("Provider mismatch".to_string()));
        }

        // Delete used state
        self.state_storage.delete(state).await?;

        // Exchange code for tokens
        let provider = self.get_provider(provider_name)?;
        let tokens = provider.exchange_code(code).await?;

        // Get user info
        let user_info = provider.get_user_info(&tokens.access_token).await?;

        // Find or create user
        let (user, is_new) = if let Some(link_user_id) = stored_state.link_user_id {
            // Link to existing user
            let user = self
                .user_repository
                .find_by_id(link_user_id)
                .await?
                .ok_or(AuthError::UserNotFound)?;
            (user, false)
        } else {
            // Find by linked identity
            if let Some(linked) = self
                .identity_storage
                .find_by_provider(&user_info.provider, &user_info.provider_id)
                .await?
            {
                let user = self
                    .user_repository
                    .find_by_id(linked.user_id)
                    .await?
                    .ok_or(AuthError::UserNotFound)?;
                (user, false)
            } else if let Some(email) = &user_info.email {
                // Try to find by email
                if let Some(user) = self.user_repository.find_by_email(email).await? {
                    (user, false)
                } else {
                    // Create new user
                    let user = self.create_user_from_oauth(&user_info).await?;
                    (user, true)
                }
            } else {
                // Create new user without email
                let user = self.create_user_from_oauth(&user_info).await?;
                (user, true)
            }
        };

        // Save or update linked identity
        let linked = LinkedIdentity {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user.id,
            provider: user_info.provider.clone(),
            provider_id: user_info.provider_id.clone(),
            email: user_info.email.clone(),
            access_token: Some(tokens.access_token),
            refresh_token: tokens.refresh_token,
            token_expires_at: tokens.expires_in.map(|secs| Utc::now() + Duration::seconds(secs as i64)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.identity_storage.upsert(&linked).await?;

        // Emit event
        self.event_emitter
            .emit(AuthEvent::OAuth2Login {
                user_id: user.id,
                provider: provider_name.to_string(),
                is_new_user: is_new,
                timestamp: Utc::now(),
            })
            .await;

        info!(user_id = %user.id, is_new = %is_new, "OAuth2 authentication successful");

        Ok(user.to_identity(AuthMethod::OAuth2, None))
    }

    /// Create user from OAuth2 info
    async fn create_user_from_oauth(&self, info: &OAuth2UserInfo) -> AuthResult<User> {
        let username = info
            .username
            .clone()
            .or_else(|| info.email.as_ref().map(|e| e.split('@').next().unwrap_or("user").to_string()))
            .unwrap_or_else(|| format!("{}_{}", info.provider, &info.provider_id[..8.min(info.provider_id.len())]));

        let mut user = User::new(&username);
        user.email = info.email.clone();
        user.email_verified = info.email_verified;
        user.display_name = info.name.clone();
        user.roles.insert("user".to_string());

        self.user_repository.create(&user).await?;

        Ok(user)
    }

    /// Get provider by name
    fn get_provider(&self, name: &str) -> AuthResult<&Arc<dyn OAuth2ProviderTrait>> {
        self.providers
            .get(name)
            .ok_or_else(|| AuthError::OAuth2Error(format!("Unknown provider: {}", name)))
    }

    /// Get linked identities for a user
    pub async fn get_linked_identities(&self, user_id: UserId) -> AuthResult<Vec<LinkedIdentity>> {
        self.identity_storage.get_user_identities(user_id).await
    }

    /// Unlink an identity
    pub async fn unlink_identity(&self, user_id: UserId, provider: &str) -> AuthResult<()> {
        self.identity_storage.delete(user_id, provider).await?;

        self.event_emitter
            .emit(AuthEvent::OAuth2Unlinked {
                user_id,
                provider: provider.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }
}

/// OAuth2 state storage trait
#[async_trait]
pub trait OAuth2StateStorage: Send + Sync {
    async fn store(&self, state: &OAuth2State) -> AuthResult<()>;
    async fn get(&self, state: &str) -> AuthResult<Option<OAuth2State>>;
    async fn delete(&self, state: &str) -> AuthResult<()>;
}

/// Linked identity storage trait
#[async_trait]
pub trait LinkedIdentityStorage: Send + Sync {
    async fn upsert(&self, identity: &LinkedIdentity) -> AuthResult<()>;
    async fn find_by_provider(&self, provider: &str, provider_id: &str) -> AuthResult<Option<LinkedIdentity>>;
    async fn get_user_identities(&self, user_id: UserId) -> AuthResult<Vec<LinkedIdentity>>;
    async fn delete(&self, user_id: UserId, provider: &str) -> AuthResult<()>;
}

/// In-memory implementations
pub struct InMemoryOAuth2StateStorage {
    states: RwLock<HashMap<String, OAuth2State>>,
}

impl InMemoryOAuth2StateStorage {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl OAuth2StateStorage for InMemoryOAuth2StateStorage {
    async fn store(&self, state: &OAuth2State) -> AuthResult<()> {
        let mut states = self.states.write().await;
        states.insert(state.state.clone(), state.clone());
        Ok(())
    }

    async fn get(&self, state: &str) -> AuthResult<Option<OAuth2State>> {
        let states = self.states.read().await;
        Ok(states.get(state).cloned())
    }

    async fn delete(&self, state: &str) -> AuthResult<()> {
        let mut states = self.states.write().await;
        states.remove(state);
        Ok(())
    }
}

pub struct InMemoryLinkedIdentityStorage {
    identities: RwLock<Vec<LinkedIdentity>>,
}

impl InMemoryLinkedIdentityStorage {
    pub fn new() -> Self {
        Self {
            identities: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl LinkedIdentityStorage for InMemoryLinkedIdentityStorage {
    async fn upsert(&self, identity: &LinkedIdentity) -> AuthResult<()> {
        let mut identities = self.identities.write().await;
        identities.retain(|i| !(i.user_id == identity.user_id && i.provider == identity.provider));
        identities.push(identity.clone());
        Ok(())
    }

    async fn find_by_provider(&self, provider: &str, provider_id: &str) -> AuthResult<Option<LinkedIdentity>> {
        let identities = self.identities.read().await;
        Ok(identities
            .iter()
            .find(|i| i.provider == provider && i.provider_id == provider_id)
            .cloned())
    }

    async fn get_user_identities(&self, user_id: UserId) -> AuthResult<Vec<LinkedIdentity>> {
        let identities = self.identities.read().await;
        Ok(identities.iter().filter(|i| i.user_id == user_id).cloned().collect())
    }

    async fn delete(&self, user_id: UserId, provider: &str) -> AuthResult<()> {
        let mut identities = self.identities.write().await;
        identities.retain(|i| !(i.user_id == user_id && i.provider == provider));
        Ok(())
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_state_generation() {
        let state1 = OAuth2State::new("github");
        let state2 = OAuth2State::new("github");

        assert_ne!(state1.state, state2.state);
        assert_eq!(state1.state.len(), 32);
        assert!(!state1.is_expired());
    }

    #[test]
    fn test_oauth2_state_expiration() {
        let mut state = OAuth2State::new("github");
        state.created_at = Utc::now() - Duration::minutes(15);

        assert!(state.is_expired());
    }

    #[tokio::test]
    async fn test_state_storage() {
        let storage = InMemoryOAuth2StateStorage::new();
        let state = OAuth2State::new("github");

        storage.store(&state).await.unwrap();
        let retrieved = storage.get(&state.state).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().provider, "github");
    }

    #[tokio::test]
    async fn test_linked_identity_storage() {
        let storage = InMemoryLinkedIdentityStorage::new();
        let user_id = UserId::new();

        let identity = LinkedIdentity {
            id: "test".to_string(),
            user_id,
            provider: "github".to_string(),
            provider_id: "12345".to_string(),
            email: Some("test@example.com".to_string()),
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        storage.upsert(&identity).await.unwrap();

        let found = storage
            .find_by_provider("github", "12345")
            .await
            .unwrap();
        assert!(found.is_some());

        let user_identities = storage.get_user_identities(user_id).await.unwrap();
        assert_eq!(user_identities.len(), 1);
    }

    #[test]
    fn test_github_authorization_url() {
        let provider = GitHubOAuth2Provider::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        let url = provider.authorization_url("test_state");

        assert!(url.contains("github.com"));
        assert!(url.contains("client_id=client_id"));
        assert!(url.contains("state=test_state"));
    }

    #[test]
    fn test_google_authorization_url() {
        let provider = GoogleOAuth2Provider::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        let url = provider.authorization_url("test_state");

        assert!(url.contains("accounts.google.com"));
        assert!(url.contains("client_id=client_id"));
        assert!(url.contains("scope="));
        assert!(url.contains("state=test_state"));
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthIdentity and AuthCredentials
- **Spec 367**: Auth Configuration - Uses OAuth2ProviderConfig
- **Spec 368**: Local Auth - Alternative authentication method
- **Spec 381**: Audit Logging - Logs OAuth2 events
- **Spec 384**: Auth Events - Emits OAuth2 events
