use anyhow::{anyhow, Result};
use chrono::Datelike;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::{SmtpTransport, Transport};
use tera::{Context, Tera};

use crate::models::Crate;

pub struct HtmlMailer {
    mailer: SmtpTransport,
    template_engine: Tera,
    smtp_username: String,
}

impl HtmlMailer {
    pub fn new(
        template_engine: Tera,
        host: String,
        username: String,
        password: String,
    ) -> Result<Self> {
        // ---
        let mailer = SmtpTransport::relay(&host)?
            .credentials(Credentials::new(username.clone(), password))
            .build();

        Ok(Self {
            mailer,
            template_engine,
            smtp_username: username,
        })
    }

    pub fn send_digest(&self, email: &str, crates: &[Crate]) -> Result<()> {
        // --
        if !crates.is_empty() {
            println!("Sending digest for {} crates", crates.len());
            let year = chrono::Utc::now().year();

            let mut context = Context::new();
            context.insert("crates", crates);
            context.insert("year", &year);

            let body = self
                .template_engine
                .render("email/digest.html", &context)
                .map_err(|e| anyhow!("Failed to render email template: {}", e))?;

            let message = Message::builder()
                .from(self.smtp_username.parse()?)
                .to(email.parse()?)
                .subject("Your Crate Digest")
                .body(body)
                .map_err(|e| anyhow!("Failed to build email message: {}", e))?;

            self.mailer
                .send(&message)
                .map(|_res| ()) // discard lettre::Response
                .map_err(|e| anyhow!("Failed to send email: {}", e))?;
        }

        Ok(())
    }
}
