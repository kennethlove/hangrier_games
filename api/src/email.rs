//! Email sending abstraction.
//!
//! Supports two backends:
//! - **Mailpit** (dev): SMTP to `localhost:1025`, viewable at `http://localhost:8025`
//! - **Resend** (prod): REST API via `resend-rs`
//!
//! Backend chosen at startup based on `EMAIL_BACKEND` env var
//! (default: `mailpit` in dev, `resend` in prod).

use confirm_email::{generate_token, validate_token};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
};
use std::sync::LazyLock;

/// Secret key for signing email verification tokens.
/// Set via `EMAIL_SECRET` env var. Falls back to JWT secret if unset.
static EMAIL_SECRET: LazyLock<String> = LazyLock::new(|| {
    std::env::var("EMAIL_SECRET")
        .or_else(|_| std::env::var("SURREAL_JWT_SECRET"))
        .unwrap_or_else(|_| {
            tracing::warn!(
                "EMAIL_SECRET not set, using hardcoded fallback (INSECURE for production)"
            );
            "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA"
                .to_string()
        })
});

/// The sender address for outgoing emails.
static EMAIL_FROM: LazyLock<String> = LazyLock::new(|| {
    std::env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@hangrier.fun".to_string())
});

/// Generate a self-contained email verification token.
///
/// Uses AES-GCM encryption via `confirm-email` crate.
/// Token contains the email address + 24-hour expiry — no DB needed.
pub fn generate_verification_token(email: &str) -> Result<String, confirm_email::error::Error> {
    generate_token(email.to_string(), EMAIL_SECRET.clone())
}

/// Validate a verification token and return the verified email address.
pub fn validate_verification_token(token: &str) -> Result<String, confirm_email::error::Error> {
    validate_token(token.to_string(), EMAIL_SECRET.clone())
}

/// Send a verification email via the configured backend.
///
/// In dev mode, sends to Mailpit via SMTP (localhost:1025).
/// In production, sends via Resend API.
pub async fn send_verification_email(to: &str, token: &str) -> Result<(), EmailError> {
    let backend = std::env::var("EMAIL_BACKEND").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "mailpit".into()
        } else {
            "resend".into()
        }
    });

    let verify_url = format!(
        "{}/auth/verify-email?token={}",
        std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
        token
    );

    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Verify your email</title></head>
<body style="font-family: sans-serif; padding: 20px;">
    <h1>Welcome to Hangrier Games!</h1>
    <p>Thanks for signing up. Click the link below to verify your email address:</p>
    <p><a href="{}" style="display: inline-block; padding: 12px 24px; background: #4f46e5; color: white; text-decoration: none; border-radius: 6px;">Verify Email</a></p>
    <p>Or copy this link into your browser:</p>
    <p style="word-break: break-all; color: #666;">{}</p>
    <p>This link expires in 24 hours.</p>
    <p>If you didn't create an account, you can ignore this email.</p>
</body>
</html>"#,
        verify_url, verify_url
    );

    match backend.as_str() {
        "mailpit" => {
            send_via_mailpit(
                to,
                &EMAIL_FROM,
                "Verify your Hangrier Games account",
                &html_body,
            )
            .await
        }
        "resend" => {
            send_via_resend(
                to,
                &EMAIL_FROM,
                "Verify your Hangrier Games account",
                &html_body,
            )
            .await
        }
        other => {
            tracing::warn!("Unknown EMAIL_BACKEND '{}', falling back to mailpit", other);
            send_via_mailpit(
                to,
                &EMAIL_FROM,
                "Verify your Hangrier Games account",
                &html_body,
            )
            .await
        }
    }
}

/// Send a password reset email via the configured backend.
pub async fn send_password_reset_email(to: &str, token: &str) -> Result<(), EmailError> {
    let backend = std::env::var("EMAIL_BACKEND").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "mailpit".into()
        } else {
            "resend".into()
        }
    });

    let reset_url = format!(
        "{}/auth/reset-password?token={}",
        std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
        token
    );

    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Reset your password</title></head>
<body style="font-family: sans-serif; padding: 20px;">
    <h1>Password Reset</h1>
    <p>We received a request to reset the password for your Hangrier Games account.</p>
    <p><a href="{}" style="display: inline-block; padding: 12px 24px; background: #4f46e5; color: white; text-decoration: none; border-radius: 6px;">Reset Password</a></p>
    <p>Or copy this link into your browser:</p>
    <p style="word-break: break-all; color: #666;">{}</p>
    <p>This link expires in 24 hours.</p>
    <p>If you didn't request a password reset, you can ignore this email.</p>
</body>
</html>"#,
        reset_url, reset_url
    );

    match backend.as_str() {
        "mailpit" => {
            send_via_mailpit(
                to,
                &EMAIL_FROM,
                "Reset your Hangrier Games password",
                &html_body,
            )
            .await
        }
        "resend" => {
            send_via_resend(
                to,
                &EMAIL_FROM,
                "Reset your Hangrier Games password",
                &html_body,
            )
            .await
        }
        other => {
            tracing::warn!("Unknown EMAIL_BACKEND '{}', falling back to mailpit", other);
            send_via_mailpit(
                to,
                &EMAIL_FROM,
                "Reset your Hangrier Games password",
                &html_body,
            )
            .await
        }
    }
}

async fn send_via_mailpit(
    to: &str,
    from: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), EmailError> {
    let smtp_host = std::env::var("MAILPIT_HOST").unwrap_or_else(|_| "localhost".to_string());
    let smtp_port: u16 = std::env::var("MAILPIT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1025);

    let email = Message::builder()
        .from(
            from.parse()
                .map_err(|e| EmailError::Build(format!("Invalid from address: {}", e)))?,
        )
        .to(to
            .parse()
            .map_err(|e| EmailError::Build(format!("Invalid to address: {}", e)))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| EmailError::Build(e.to_string()))?;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_host)
        .port(smtp_port)
        .build();

    mailer
        .send(email)
        .await
        .map_err(|e| EmailError::Send(e.to_string()))?;

    tracing::info!(
        "[Mailpit] Verification email sent to {} — view at http://{}:8025",
        to,
        smtp_host
    );

    Ok(())
}

async fn send_via_resend(
    to: &str,
    from: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), EmailError> {
    let api_key = std::env::var("RESEND_API_KEY")
        .map_err(|_| EmailError::Config("RESEND_API_KEY not set".into()))?;

    let client = resend_rs::Resend::new(&api_key);

    let email =
        resend_rs::types::CreateEmailBaseOptions::new(from, [to], subject).with_html(html_body);

    client
        .emails
        .send(email)
        .await
        .map_err(|e| EmailError::Send(e.to_string()))?;

    tracing::info!("[Resend] Verification email sent to {}", to);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Email configuration error: {0}")]
    Config(String),
    #[error("Failed to build email: {0}")]
    Build(String),
    #[error("Failed to send email: {0}")]
    Send(String),
}
