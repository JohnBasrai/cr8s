//! Tera-based HTML email renderer for cr8s.
//!
//! Internal implementation of `MailerTrait` using HTML templates.
//! Do not use directly — access via `crate::mail::create_mailer()`.
//!
//! Converts domain email templates into final HTML bodies using the Tera engine.
//! This file does not send mail—it focuses only on formatting.

use crate::domain::{CrateSummary, MailerTrait, MailerTraitPtr};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};
use tera::{Context, Tera};

#[derive(Clone)]
pub struct HtmlMailer {
    tera: Tera,
    smtp_host: String,
    smtp_user: String,
    smtp_pass: String,
}

impl HtmlMailer {
    // ---
    fn new(tera: Tera, smtp_host: String, smtp_user: String, smtp_pass: String) -> Result<Self> {
        Ok(Self {
            tera,
            smtp_host,
            smtp_user,
            smtp_pass,
        })
    }
}

impl HtmlMailer {
    // --
    fn send_digest_blocking(&self, to: &str, crates: &[CrateSummary]) -> Result<()> {
        // ---
        let mut context = Context::new();
        context.insert("crates", crates); // Make sure `CrateSummary: Serialize`

        let html = self.tera.render("digest.html", &context)?;

        let email = Message::builder()
            .from(self.smtp_user.parse::<Mailbox>()?)
            .to(to.parse::<Mailbox>()?)
            .subject("📦 Crate Digest")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html)?;

        let creds = lettre::transport::smtp::authentication::Credentials::new(
            self.smtp_user.clone(),
            self.smtp_pass.clone(),
        );

        let mailer = SmtpTransport::relay(&self.smtp_host)?
            .credentials(creds)
            .build();

        mailer
            .send(&email)
            .map_err(|e| anyhow!("SMTP send failed: {e:?}"))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl MailerTrait for HtmlMailer {
    // --
    async fn send_digest(&self, dest: &str, crates: &[CrateSummary]) -> Result<()> {
        // --
        let this = self.clone();
        let dest = dest.to_string();
        let crates = crates.to_owned(); // Clone the crates to make them owned

        tokio::task::spawn_blocking(move || this.send_digest_blocking(&dest, &crates)).await??;

        Ok(())
    }
}

/// Constructs the default mailer using Tera templates and SMTP credentials.
///
/// Loads all HTML templates under `templates/**/*.html`, then reads the following
/// environment variables to configure the SMTP transport:
/// - `SMTP_HOST`
/// - `SMTP_USERNAME`
/// - `SMTP_PASSWORD`
///
/// Returns a trait object (`MailerTraitPtr`) wrapped in an `Arc`.
///
/// # Errors
/// Returns an error if templates cannot be loaded or any required env vars are missing.
pub fn create_mailer() -> Result<MailerTraitPtr> {
    // ---
    let tera = Tera::new("templates/**/*.html").context("Cannot load template engine")?;
    let hostname = std::env::var("SMTP_HOST").context("Missing SMTP_HOST env var")?;
    let username = std::env::var("SMTP_USERNAME").context("Missing SMTP_USERNAME env var")?;
    let password = std::env::var("SMTP_PASSWORD").context("Missing SMTP_PASSWORD env var")?;

    Ok(std::sync::Arc::new(HtmlMailer::new(
        tera, hostname, username, password,
    )?))
}
