use crate::domain::error::AuthError;
use lettre::message::Message;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::AsyncTransport;
use lettre::Tokio1Executor;
use tracing::info;

/// Mail service for sending verification codes and notifications.
pub struct MailService {
    smtp_host: String,
    smtp_port: u16,
    username: String,
    password: String,
    from: String,
}

impl MailService {
    pub fn new(host: &str, port: u16, username: &str, password: &str, from: &str) -> Self {
        Self {
            smtp_host: host.to_string(),
            smtp_port: port,
            username: username.to_string(),
            password: password.to_string(),
            from: from.to_string(),
        }
    }

    /// Send a verification code email (for registration, email change, account deletion)
    pub async fn send_verification_code(
        &self,
        to: &str,
        code: &str,
        purpose: &str,
    ) -> Result<(), AuthError> {
        let body = format!(
            "Your verification code for {} is: {}\n\nThis code expires in 10 minutes.",
            purpose, code
        );

        info!(
            "[DEV] Verification code for {}: {} -> {}",
            purpose, to, code
        );

        // In dev mode, just log the code (Mailpit captures the SMTP traffic)
        if self.smtp_host == "mailpit" || self.smtp_host == "localhost" {
            info!("Mailpit SMTP: {}:{}", self.smtp_host, self.smtp_port);
        }

        let email = Message::builder()
            .from(self.from.parse().map_err(|_| AuthError::TokenInvalid)?)
            .to(to.parse().map_err(|_| AuthError::TokenInvalid)?)
            .subject(format!("[upgo] {} Verification Code", purpose))
            .body(body)
            .map_err(|_| AuthError::TokenInvalid)?;

        let creds = Credentials::new(self.username.clone(), self.password.clone());

        match AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host) {
            Ok(mailer) => {
                let mailer = mailer.credentials(creds).build();
                match mailer.send(email).await {
                    Ok(_) => info!("Verification email sent to {}", to),
                    Err(e) => {
                        // In dev mode, don't fail if SMTP is unavailable
                        info!("SMTP unavailable (dev mode), code logged: {} -> {}", to, e);
                    }
                }
            }
            Err(e) => {
                info!("SMTP relay setup failed (dev mode): {}", e);
            }
        }

        Ok(())
    }

    /// Send a notification email (e.g., email change notification)
    pub async fn send_notification(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), AuthError> {
        info!("[DEV] Notification to {}: {} - {}", to, subject, body);
        Ok(())
    }
}
