//! Email notification sender via SMTP.
//!
//! Configuration is read from environment variables:
//!   SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS, SMTP_FROM, SMTP_TLS

use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

/// Email notification sender using SMTP.
pub struct EmailNotifier {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_addr: String,
    tls: bool,
}

impl EmailNotifier {
    /// Create from environment variables. Returns None if SMTP_HOST is not set.
    pub fn from_env() -> Option<Self> {
        let host = std::env::var("SMTP_HOST").ok()?;
        let port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let username = std::env::var("SMTP_USER").unwrap_or_default();
        let password = std::env::var("SMTP_PASS").unwrap_or_default();
        let from_addr = std::env::var("SMTP_FROM").unwrap_or_else(|_| "opspilot@localhost".into());
        let tls = std::env::var("SMTP_TLS")
            .ok()
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        Some(Self { host, port, username, password, from_addr, tls })
    }

    /// Send an email to one or more recipients.
    pub async fn send(&self, to: &[String], subject: &str, body: &str) -> anyhow::Result<()> {
        let from_mailbox: Mailbox = self.from_addr.parse()
            .map_err(|e| anyhow::anyhow!("invalid from address: {}", e))?;

        let mut email_builder = Message::builder()
            .from(from_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN);

        for addr in to {
            let mailbox: Mailbox = addr.parse()
                .map_err(|e| anyhow::anyhow!("invalid email address '{}': {}", addr, e))?;
            email_builder = email_builder.to(mailbox);
        }

        let email = email_builder.body(body.to_string())
            .map_err(|e| anyhow::anyhow!("failed to build email: {}", e))?;

        let credentials = if !self.username.is_empty() {
            Some(Credentials::new(self.username.clone(), self.password.clone()))
        } else {
            None
        };

        let transport = if self.tls {
            let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)
                .map_err(|e| anyhow::anyhow!("SMTP relay error: {}", e))?;
            if let Some(creds) = credentials {
                builder = builder.credentials(creds);
            }
            builder.build()
        } else {
            let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host)
                .port(self.port);
            if let Some(creds) = credentials {
                builder = builder.credentials(creds);
            }
            builder.build()
        };

        transport.send(email).await
            .map_err(|e| anyhow::anyhow!("SMTP send error: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_missing_host() {
        std::env::remove_var("SMTP_HOST");
        assert!(EmailNotifier::from_env().is_none());
    }

    #[test]
    fn test_from_env_with_host() {
        std::env::set_var("SMTP_HOST", "smtp.example.com");
        std::env::set_var("SMTP_PORT", "465");
        std::env::set_var("SMTP_USER", "user");
        std::env::set_var("SMTP_PASS", "pass");
        std::env::set_var("SMTP_FROM", "ops@example.com");

        let notifier = EmailNotifier::from_env().unwrap();
        assert_eq!(notifier.host, "smtp.example.com");
        assert_eq!(notifier.port, 465);
        assert_eq!(notifier.from_addr, "ops@example.com");

        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_PORT");
        std::env::remove_var("SMTP_USER");
        std::env::remove_var("SMTP_PASS");
        std::env::remove_var("SMTP_FROM");
    }
}
