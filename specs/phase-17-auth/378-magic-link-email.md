# Spec 378: Magic Link Email

## Overview
Implement email sending for magic link authentication with template support.

## Rust Implementation

### Magic Link Email Service
```rust
// src/auth/magic_link/email.rs

use super::types::*;
use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use lettre::message::{header::ContentType, MultiPart, SinglePart};
use tracing::{debug, info, warn, instrument};

/// Email configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// SMTP host
    pub smtp_host: String,
    /// SMTP port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_username: Option<String>,
    /// SMTP password
    pub smtp_password: Option<String>,
    /// Use TLS
    pub use_tls: bool,
    /// Sender email address
    pub sender_email: String,
    /// Sender name
    pub sender_name: String,
    /// Reply-to address
    pub reply_to: Option<String>,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: None,
            smtp_password: None,
            use_tls: true,
            sender_email: "noreply@tachikoma.local".to_string(),
            sender_name: "Tachikoma".to_string(),
            reply_to: None,
        }
    }
}

/// Email template engine
pub struct EmailTemplates {
    /// App name for templates
    pub app_name: String,
    /// Support email
    pub support_email: Option<String>,
    /// Base URL
    pub base_url: String,
}

impl EmailTemplates {
    pub fn new(app_name: &str, base_url: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            support_email: None,
            base_url: base_url.to_string(),
        }
    }

    /// Generate magic link email HTML
    pub fn magic_link_html(&self, data: &MagicLinkEmailData) -> String {
        let action_text = match data.purpose.as_str() {
            "login" => "Sign in to your account",
            "signup" => "Create your account",
            "email_verification" => "Verify your email",
            "password_reset" => "Reset your password",
            "account_deletion" => "Confirm account deletion",
            _ => "Continue",
        };

        let warning_text = if data.purpose == "account_deletion" {
            r#"<p style="color: #dc2626; font-weight: bold;">Warning: This action is irreversible.</p>"#
        } else {
            ""
        };

        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{action_text} - {app_name}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; border-radius: 10px 10px 0 0;">
        <h1 style="color: white; margin: 0; font-size: 24px;">{app_name}</h1>
    </div>

    <div style="background: #ffffff; padding: 30px; border: 1px solid #e5e7eb; border-top: none; border-radius: 0 0 10px 10px;">
        <h2 style="margin-top: 0; color: #1f2937;">{action_text}</h2>

        <p>Hello{recipient_greeting},</p>

        <p>Click the button below to {action_lower}. This link will expire in {expires_in_minutes} minutes.</p>

        {warning_text}

        <div style="text-align: center; margin: 30px 0;">
            <a href="{magic_link_url}" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 14px 30px; text-decoration: none; border-radius: 6px; font-weight: bold; display: inline-block;">
                {action_text}
            </a>
        </div>

        <p style="font-size: 14px; color: #6b7280;">
            If you didn't request this email, you can safely ignore it.
        </p>

        <p style="font-size: 14px; color: #6b7280;">
            Or copy and paste this link into your browser:<br>
            <a href="{magic_link_url}" style="color: #667eea; word-break: break-all;">{magic_link_url}</a>
        </p>

        <hr style="border: none; border-top: 1px solid #e5e7eb; margin: 20px 0;">

        <div style="font-size: 12px; color: #9ca3af;">
            <p>This request was made from:</p>
            <ul style="margin: 5px 0; padding-left: 20px;">
                {ip_info}
                {device_info}
            </ul>
        </div>
    </div>

    <div style="text-align: center; padding: 20px; font-size: 12px; color: #9ca3af;">
        <p>&copy; {year} {app_name}. All rights reserved.</p>
    </div>
</body>
</html>
"#,
            app_name = self.app_name,
            action_text = action_text,
            action_lower = action_text.to_lowercase(),
            recipient_greeting = data.recipient_name.as_ref().map(|n| format!(" {}", n)).unwrap_or_default(),
            expires_in_minutes = data.expires_in_minutes,
            magic_link_url = data.magic_link_url,
            warning_text = warning_text,
            ip_info = data.ip_address.as_ref().map(|ip| format!("<li>IP Address: {}</li>", ip)).unwrap_or_default(),
            device_info = data.user_agent.as_ref().map(|ua| format!("<li>Device: {}</li>", ua)).unwrap_or_default(),
            year = chrono::Utc::now().format("%Y"),
        )
    }

    /// Generate magic link email plain text
    pub fn magic_link_text(&self, data: &MagicLinkEmailData) -> String {
        let action_text = match data.purpose.as_str() {
            "login" => "Sign in to your account",
            "signup" => "Create your account",
            "email_verification" => "Verify your email",
            "password_reset" => "Reset your password",
            "account_deletion" => "Confirm account deletion",
            _ => "Continue",
        };

        format!(r#"
{app_name}

{action_text}

Hello{recipient_greeting},

Click the link below to {action_lower}. This link will expire in {expires_in_minutes} minutes.

{magic_link_url}

If you didn't request this email, you can safely ignore it.

This request was made from:
- IP Address: {ip_address}
- Device: {user_agent}

---
{app_name}
"#,
            app_name = self.app_name,
            action_text = action_text,
            action_lower = action_text.to_lowercase(),
            recipient_greeting = data.recipient_name.as_ref().map(|n| format!(" {}", n)).unwrap_or_default(),
            expires_in_minutes = data.expires_in_minutes,
            magic_link_url = data.magic_link_url,
            ip_address = data.ip_address.as_deref().unwrap_or("Unknown"),
            user_agent = data.user_agent.as_deref().unwrap_or("Unknown"),
        )
    }

    /// Get email subject for purpose
    pub fn magic_link_subject(&self, purpose: &str) -> String {
        match purpose {
            "login" => format!("Sign in to {} ", self.app_name),
            "signup" => format!("Welcome to {}! Confirm your email", self.app_name),
            "email_verification" => format!("Verify your email for {}", self.app_name),
            "password_reset" => format!("Reset your {} password", self.app_name),
            "account_deletion" => format!("Confirm account deletion - {}", self.app_name),
            _ => format!("{} - Action Required", self.app_name),
        }
    }
}

/// Magic link email sender
pub struct MagicLinkEmailSender {
    config: EmailConfig,
    templates: EmailTemplates,
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl MagicLinkEmailSender {
    pub fn new(config: EmailConfig, templates: EmailTemplates) -> Result<Self, MagicLinkError> {
        let transport = if config.smtp_host != "none" {
            let builder = if config.use_tls {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
                    .map_err(|e| MagicLinkError::EmailFailed(e.to_string()))?
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
            };

            let mut builder = builder.port(config.smtp_port);

            if let (Some(username), Some(password)) = (&config.smtp_username, &config.smtp_password) {
                builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
            }

            Some(builder.build())
        } else {
            None
        };

        Ok(Self {
            config,
            templates,
            transport,
        })
    }

    /// Create a test sender (logs instead of sending)
    pub fn test(templates: EmailTemplates) -> Self {
        Self {
            config: EmailConfig {
                smtp_host: "none".to_string(),
                ..Default::default()
            },
            templates,
            transport: None,
        }
    }

    /// Send magic link email
    #[instrument(skip(self, data))]
    pub async fn send(&self, data: &MagicLinkEmailData) -> Result<(), MagicLinkError> {
        let subject = self.templates.magic_link_subject(&data.purpose);
        let html = self.templates.magic_link_html(data);
        let text = self.templates.magic_link_text(data);

        // Build message
        let from = format!("{} <{}>", self.config.sender_name, self.config.sender_email)
            .parse()
            .map_err(|e| MagicLinkError::EmailFailed(format!("Invalid from address: {}", e)))?;

        let to = data.recipient_email.parse()
            .map_err(|e| MagicLinkError::EmailFailed(format!("Invalid to address: {}", e)))?;

        let mut builder = Message::builder()
            .from(from)
            .to(to)
            .subject(&subject);

        if let Some(reply_to) = &self.config.reply_to {
            builder = builder.reply_to(reply_to.parse()
                .map_err(|e| MagicLinkError::EmailFailed(format!("Invalid reply-to address: {}", e)))?);
        }

        let message = builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text)
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html)
                    )
            )
            .map_err(|e| MagicLinkError::EmailFailed(e.to_string()))?;

        // Send or log
        match &self.transport {
            Some(transport) => {
                transport.send(message).await
                    .map_err(|e| MagicLinkError::EmailFailed(e.to_string()))?;

                info!("Sent magic link email to {}", data.recipient_email);
            }
            None => {
                info!(
                    "Magic link email (test mode): to={}, url={}",
                    data.recipient_email,
                    data.magic_link_url
                );
            }
        }

        Ok(())
    }

    /// Send with retry
    pub async fn send_with_retry(&self, data: &MagicLinkEmailData, max_retries: u32) -> Result<(), MagicLinkError> {
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.send(data).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    warn!("Email send attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    if attempt + 1 < max_retries {
                        tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(MagicLinkError::EmailFailed("Unknown error".to_string())))
    }
}

/// Email queue for async sending
pub struct EmailQueue {
    sender: MagicLinkEmailSender,
    queue: tokio::sync::mpsc::Sender<MagicLinkEmailData>,
}

impl EmailQueue {
    pub fn new(sender: MagicLinkEmailSender, buffer_size: usize) -> (Self, EmailQueueWorker) {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer_size);

        let queue = Self {
            sender: sender.clone(),
            queue: tx,
        };

        let worker = EmailQueueWorker {
            sender,
            receiver: rx,
        };

        (queue, worker)
    }

    /// Queue an email for sending
    pub async fn enqueue(&self, data: MagicLinkEmailData) -> Result<(), MagicLinkError> {
        self.queue.send(data).await
            .map_err(|e| MagicLinkError::EmailFailed(format!("Queue full: {}", e)))
    }

    /// Send immediately (bypass queue)
    pub async fn send_now(&self, data: &MagicLinkEmailData) -> Result<(), MagicLinkError> {
        self.sender.send(data).await
    }
}

impl Clone for MagicLinkEmailSender {
    fn clone(&self) -> Self {
        // Note: We can't clone the transport, so create a new one
        Self::new(self.config.clone(), EmailTemplates {
            app_name: self.templates.app_name.clone(),
            support_email: self.templates.support_email.clone(),
            base_url: self.templates.base_url.clone(),
        }).unwrap_or_else(|_| Self {
            config: self.config.clone(),
            templates: EmailTemplates {
                app_name: self.templates.app_name.clone(),
                support_email: self.templates.support_email.clone(),
                base_url: self.templates.base_url.clone(),
            },
            transport: None,
        })
    }
}

/// Email queue worker
pub struct EmailQueueWorker {
    sender: MagicLinkEmailSender,
    receiver: tokio::sync::mpsc::Receiver<MagicLinkEmailData>,
}

impl EmailQueueWorker {
    pub async fn run(mut self) {
        info!("Email queue worker started");

        while let Some(data) = self.receiver.recv().await {
            if let Err(e) = self.sender.send_with_retry(&data, 3).await {
                warn!("Failed to send email after retries: {}", e);
            }
        }

        info!("Email queue worker stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_templates() {
        let templates = EmailTemplates::new("TestApp", "http://localhost:8080");

        let data = MagicLinkEmailData {
            recipient_email: "test@example.com".to_string(),
            recipient_name: Some("Test User".to_string()),
            magic_link_url: "http://localhost:8080/verify?token=abc123".to_string(),
            purpose: "login".to_string(),
            expires_in_minutes: 15,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            app_name: "TestApp".to_string(),
        };

        let html = templates.magic_link_html(&data);
        assert!(html.contains("Sign in to your account"));
        assert!(html.contains("abc123"));
        assert!(html.contains("15 minutes"));

        let text = templates.magic_link_text(&data);
        assert!(text.contains("Sign in"));
        assert!(text.contains("abc123"));
    }

    #[test]
    fn test_subject_generation() {
        let templates = EmailTemplates::new("MyApp", "http://localhost");

        assert!(templates.magic_link_subject("login").contains("Sign in"));
        assert!(templates.magic_link_subject("signup").contains("Welcome"));
        assert!(templates.magic_link_subject("password_reset").contains("Reset"));
    }
}
```

## Files to Create
- `src/auth/magic_link/email.rs` - Email sending service
- `src/auth/magic_link/templates.rs` - Email templates
