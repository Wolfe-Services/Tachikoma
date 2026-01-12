# Spec 384: Authentication UI

## Overview
Implement UI components and routes for authentication flows including login, registration, and OAuth callbacks.

## Rust Implementation

### Auth UI Routes
```rust
// src/auth/ui.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, instrument};

use super::oauth::github::{GitHubOAuthService, OAuthCallback};
use super::oauth::google::GoogleOAuthService;
use super::magic_link::MagicLinkService;

/// Auth UI state
pub struct AuthUiState {
    pub templates: AuthTemplates,
    pub github_oauth: Option<Arc<GitHubOAuthService>>,
    pub google_oauth: Option<Arc<GoogleOAuthService>>,
    pub magic_link: Option<Arc<MagicLinkService>>,
    pub config: AuthUiConfig,
}

/// UI configuration
#[derive(Debug, Clone)]
pub struct AuthUiConfig {
    pub app_name: String,
    pub logo_url: Option<String>,
    pub background_color: String,
    pub primary_color: String,
    pub success_redirect: String,
    pub login_url: String,
    pub enable_github: bool,
    pub enable_google: bool,
    pub enable_magic_link: bool,
    pub enable_password: bool,
}

impl Default for AuthUiConfig {
    fn default() -> Self {
        Self {
            app_name: "Tachikoma".to_string(),
            logo_url: None,
            background_color: "#f3f4f6".to_string(),
            primary_color: "#6366f1".to_string(),
            success_redirect: "/dashboard".to_string(),
            login_url: "/auth/login".to_string(),
            enable_github: true,
            enable_google: true,
            enable_magic_link: true,
            enable_password: true,
        }
    }
}

/// Create auth UI router
pub fn auth_ui_router(state: Arc<AuthUiState>) -> Router {
    Router::new()
        // Auth pages
        .route("/login", get(login_page).post(login_submit))
        .route("/register", get(register_page).post(register_submit))
        .route("/logout", get(logout_page))
        .route("/forgot-password", get(forgot_password_page).post(forgot_password_submit))

        // OAuth callbacks
        .route("/github/callback", get(github_callback))
        .route("/google/callback", get(google_callback))

        // Magic link
        .route("/magic-link", get(magic_link_page).post(magic_link_submit))
        .route("/magic-link/verify", get(magic_link_verify))

        // Device code
        .route("/device", get(device_auth_page).post(device_auth_submit))

        // Error page
        .route("/error", get(error_page))

        .with_state(state)
}

// === Templates ===

/// HTML template engine
pub struct AuthTemplates {
    config: AuthUiConfig,
}

impl AuthTemplates {
    pub fn new(config: AuthUiConfig) -> Self {
        Self { config }
    }

    /// Base HTML wrapper
    fn base(&self, title: &str, content: &str) -> String {
        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - {app_name}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background-color: {bg_color};
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .auth-container {{
            background: white;
            border-radius: 12px;
            box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);
            padding: 40px;
            width: 100%;
            max-width: 420px;
            margin: 20px;
        }}
        .auth-header {{ text-align: center; margin-bottom: 30px; }}
        .auth-header h1 {{ font-size: 24px; color: #1f2937; margin-bottom: 8px; }}
        .auth-header p {{ color: #6b7280; font-size: 14px; }}
        .auth-form {{ display: flex; flex-direction: column; gap: 16px; }}
        .form-group {{ display: flex; flex-direction: column; gap: 6px; }}
        .form-group label {{ font-size: 14px; font-weight: 500; color: #374151; }}
        .form-group input {{
            padding: 12px;
            border: 1px solid #d1d5db;
            border-radius: 8px;
            font-size: 16px;
            transition: border-color 0.2s;
        }}
        .form-group input:focus {{ outline: none; border-color: {primary_color}; }}
        .btn {{
            padding: 12px 24px;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
        }}
        .btn-primary {{
            background-color: {primary_color};
            color: white;
        }}
        .btn-primary:hover {{ opacity: 0.9; }}
        .btn-secondary {{
            background-color: white;
            border: 1px solid #d1d5db;
            color: #374151;
        }}
        .btn-secondary:hover {{ background-color: #f9fafb; }}
        .divider {{
            display: flex;
            align-items: center;
            gap: 16px;
            color: #9ca3af;
            font-size: 14px;
        }}
        .divider::before, .divider::after {{
            content: '';
            flex: 1;
            height: 1px;
            background-color: #e5e7eb;
        }}
        .oauth-buttons {{ display: flex; flex-direction: column; gap: 12px; }}
        .oauth-btn {{
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 12px;
            padding: 12px;
            border: 1px solid #d1d5db;
            border-radius: 8px;
            background: white;
            color: #374151;
            text-decoration: none;
            font-weight: 500;
            transition: background-color 0.2s;
        }}
        .oauth-btn:hover {{ background-color: #f9fafb; }}
        .oauth-btn svg {{ width: 20px; height: 20px; }}
        .auth-footer {{ margin-top: 24px; text-align: center; font-size: 14px; color: #6b7280; }}
        .auth-footer a {{ color: {primary_color}; text-decoration: none; }}
        .auth-footer a:hover {{ text-decoration: underline; }}
        .alert {{ padding: 12px 16px; border-radius: 8px; margin-bottom: 16px; font-size: 14px; }}
        .alert-error {{ background-color: #fef2f2; color: #dc2626; border: 1px solid #fecaca; }}
        .alert-success {{ background-color: #f0fdf4; color: #16a34a; border: 1px solid #bbf7d0; }}
    </style>
</head>
<body>
    <div class="auth-container">
        {content}
    </div>
</body>
</html>
"#,
            title = title,
            app_name = self.config.app_name,
            bg_color = self.config.background_color,
            primary_color = self.config.primary_color,
            content = content
        )
    }

    /// Login page
    pub fn login(&self, error: Option<&str>, redirect: Option<&str>) -> String {
        let error_html = error.map(|e| format!(
            r#"<div class="alert alert-error">{}</div>"#, e
        )).unwrap_or_default();

        let redirect_input = redirect.map(|r| format!(
            r#"<input type="hidden" name="redirect" value="{}">"#, r
        )).unwrap_or_default();

        let oauth_html = self.oauth_buttons();

        let content = format!(r#"
            <div class="auth-header">
                <h1>Welcome back</h1>
                <p>Sign in to your account</p>
            </div>

            {error_html}

            {oauth_html}

            {divider}

            <form class="auth-form" method="POST">
                {redirect_input}
                <div class="form-group">
                    <label for="email">Email</label>
                    <input type="email" id="email" name="email" required autocomplete="email">
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input type="password" id="password" name="password" required autocomplete="current-password">
                </div>
                <button type="submit" class="btn btn-primary">Sign in</button>
            </form>

            <div class="auth-footer">
                <p>Don't have an account? <a href="/auth/register">Sign up</a></p>
                <p style="margin-top: 8px;"><a href="/auth/forgot-password">Forgot password?</a></p>
            </div>
        "#,
            error_html = error_html,
            oauth_html = oauth_html,
            redirect_input = redirect_input,
            divider = if self.has_oauth() && self.config.enable_password {
                r#"<div class="divider">or continue with email</div>"#
            } else { "" }
        );

        self.base("Sign In", &content)
    }

    /// Register page
    pub fn register(&self, error: Option<&str>) -> String {
        let error_html = error.map(|e| format!(
            r#"<div class="alert alert-error">{}</div>"#, e
        )).unwrap_or_default();

        let oauth_html = self.oauth_buttons();

        let content = format!(r#"
            <div class="auth-header">
                <h1>Create account</h1>
                <p>Get started with {app_name}</p>
            </div>

            {error_html}

            {oauth_html}

            {divider}

            <form class="auth-form" method="POST">
                <div class="form-group">
                    <label for="name">Name</label>
                    <input type="text" id="name" name="name" autocomplete="name">
                </div>
                <div class="form-group">
                    <label for="email">Email</label>
                    <input type="email" id="email" name="email" required autocomplete="email">
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input type="password" id="password" name="password" required autocomplete="new-password" minlength="8">
                </div>
                <button type="submit" class="btn btn-primary">Create account</button>
            </form>

            <div class="auth-footer">
                <p>Already have an account? <a href="/auth/login">Sign in</a></p>
            </div>
        "#,
            app_name = self.config.app_name,
            error_html = error_html,
            oauth_html = oauth_html,
            divider = if self.has_oauth() && self.config.enable_password {
                r#"<div class="divider">or sign up with email</div>"#
            } else { "" }
        );

        self.base("Sign Up", &content)
    }

    /// Magic link page
    pub fn magic_link(&self, success: bool, error: Option<&str>) -> String {
        let message = if success {
            r#"<div class="alert alert-success">Check your email for a sign-in link!</div>"#
        } else if let Some(e) = error {
            &format!(r#"<div class="alert alert-error">{}</div>"#, e)
        } else {
            ""
        };

        let content = format!(r#"
            <div class="auth-header">
                <h1>Sign in with email</h1>
                <p>We'll send you a magic link</p>
            </div>

            {message}

            <form class="auth-form" method="POST">
                <div class="form-group">
                    <label for="email">Email</label>
                    <input type="email" id="email" name="email" required autocomplete="email">
                </div>
                <button type="submit" class="btn btn-primary">Send magic link</button>
            </form>

            <div class="auth-footer">
                <p><a href="/auth/login">Back to sign in</a></p>
            </div>
        "#, message = message);

        self.base("Magic Link", &content)
    }

    /// Device authorization page
    pub fn device_auth(&self, user_code: Option<&str>, error: Option<&str>) -> String {
        let error_html = error.map(|e| format!(
            r#"<div class="alert alert-error">{}</div>"#, e
        )).unwrap_or_default();

        let code_input = user_code.map(|code| format!(
            r#"<input type="text" id="user_code" name="user_code" value="{}" required pattern="[A-Z0-9]{{4}}-[A-Z0-9]{{4}}">"#,
            code
        )).unwrap_or_else(|| {
            r#"<input type="text" id="user_code" name="user_code" required placeholder="XXXX-XXXX" pattern="[A-Z0-9]{4}-[A-Z0-9]{4}">"#.to_string()
        });

        let content = format!(r#"
            <div class="auth-header">
                <h1>Device Authorization</h1>
                <p>Enter the code shown on your device</p>
            </div>

            {error_html}

            <form class="auth-form" method="POST">
                <div class="form-group">
                    <label for="user_code">Device Code</label>
                    {code_input}
                </div>
                <button type="submit" class="btn btn-primary">Authorize Device</button>
            </form>
        "#, error_html = error_html, code_input = code_input);

        self.base("Device Authorization", &content)
    }

    /// Error page
    pub fn error(&self, title: &str, message: &str) -> String {
        let content = format!(r#"
            <div class="auth-header">
                <h1>{title}</h1>
                <p>{message}</p>
            </div>

            <div class="auth-footer">
                <p><a href="/auth/login">Back to sign in</a></p>
            </div>
        "#, title = title, message = message);

        self.base("Error", &content)
    }

    /// OAuth buttons HTML
    fn oauth_buttons(&self) -> String {
        let mut buttons = Vec::new();

        if self.config.enable_github {
            buttons.push(r#"
                <a href="/auth/github" class="oauth-btn">
                    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
                    Continue with GitHub
                </a>
            "#);
        }

        if self.config.enable_google {
            buttons.push(r#"
                <a href="/auth/google" class="oauth-btn">
                    <svg viewBox="0 0 24 24"><path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/><path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/><path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/></svg>
                    Continue with Google
                </a>
            "#);
        }

        if self.config.enable_magic_link {
            buttons.push(r#"
                <a href="/auth/magic-link" class="oauth-btn">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>
                    Continue with Email
                </a>
            "#);
        }

        if buttons.is_empty() {
            String::new()
        } else {
            format!(r#"<div class="oauth-buttons">{}</div>"#, buttons.join("\n"))
        }
    }

    fn has_oauth(&self) -> bool {
        self.config.enable_github || self.config.enable_google || self.config.enable_magic_link
    }
}

// === Handlers ===

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
    redirect: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    name: Option<String>,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct MagicLinkForm {
    email: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceAuthForm {
    user_code: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MagicLinkQuery {
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorQuery {
    message: Option<String>,
}

async fn login_page(
    State(state): State<Arc<AuthUiState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let error = params.get("error").map(String::as_str);
    let redirect = params.get("redirect").map(String::as_str);
    Html(state.templates.login(error, redirect))
}

async fn login_submit(
    State(state): State<Arc<AuthUiState>>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    // Login logic would go here
    // For now, redirect to success or show error
    let redirect_to = form.redirect.unwrap_or_else(|| state.config.success_redirect.clone());
    Redirect::to(&redirect_to)
}

async fn register_page(
    State(state): State<Arc<AuthUiState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let error = params.get("error").map(String::as_str);
    Html(state.templates.register(error))
}

async fn register_submit(
    State(state): State<Arc<AuthUiState>>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    // Registration logic would go here
    Redirect::to(&state.config.success_redirect)
}

async fn logout_page(State(state): State<Arc<AuthUiState>>) -> impl IntoResponse {
    Redirect::to(&state.config.login_url)
}

async fn forgot_password_page(State(state): State<Arc<AuthUiState>>) -> Html<String> {
    Html(state.templates.magic_link(false, None))
}

async fn forgot_password_submit(
    State(state): State<Arc<AuthUiState>>,
    Form(form): Form<MagicLinkForm>,
) -> Html<String> {
    Html(state.templates.magic_link(true, None))
}

async fn github_callback(
    State(state): State<Arc<AuthUiState>>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    if let Some(error) = query.error {
        let message = query.error_description.unwrap_or(error);
        return Redirect::to(&format!("/auth/error?message={}", urlencoding::encode(&message))).into_response();
    }

    // Handle GitHub OAuth callback
    Redirect::to(&state.config.success_redirect).into_response()
}

async fn google_callback(
    State(state): State<Arc<AuthUiState>>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    if let Some(error) = query.error {
        let message = query.error_description.unwrap_or(error);
        return Redirect::to(&format!("/auth/error?message={}", urlencoding::encode(&message))).into_response();
    }

    Redirect::to(&state.config.success_redirect).into_response()
}

async fn magic_link_page(State(state): State<Arc<AuthUiState>>) -> Html<String> {
    Html(state.templates.magic_link(false, None))
}

async fn magic_link_submit(
    State(state): State<Arc<AuthUiState>>,
    Form(form): Form<MagicLinkForm>,
) -> Html<String> {
    // Send magic link
    Html(state.templates.magic_link(true, None))
}

async fn magic_link_verify(
    State(state): State<Arc<AuthUiState>>,
    Query(query): Query<MagicLinkQuery>,
) -> impl IntoResponse {
    // Verify magic link
    Redirect::to(&state.config.success_redirect)
}

async fn device_auth_page(
    State(state): State<Arc<AuthUiState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let user_code = params.get("user_code").map(String::as_str);
    let error = params.get("error").map(String::as_str);
    Html(state.templates.device_auth(user_code, error))
}

async fn device_auth_submit(
    State(state): State<Arc<AuthUiState>>,
    Form(form): Form<DeviceAuthForm>,
) -> impl IntoResponse {
    // Authorize device
    Redirect::to(&state.config.success_redirect)
}

async fn error_page(
    State(state): State<Arc<AuthUiState>>,
    Query(query): Query<ErrorQuery>,
) -> Html<String> {
    let message = query.message.as_deref().unwrap_or("An error occurred");
    Html(state.templates.error("Error", message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates() {
        let config = AuthUiConfig::default();
        let templates = AuthTemplates::new(config);

        let login = templates.login(None, None);
        assert!(login.contains("Sign in"));
        assert!(login.contains("form"));

        let register = templates.register(None);
        assert!(register.contains("Create account"));
    }
}
```

## Files to Create
- `src/auth/ui.rs` - Authentication UI routes and templates
