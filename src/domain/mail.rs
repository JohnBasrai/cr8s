//! Trait abstraction for email delivery logic.

use crate::domain::CrateSummary;
use anyhow::Result;

/// Interface/trait for sending system notifications and digests.
#[async_trait::async_trait]
pub trait MailerTrait: Send + Sync {
    async fn send_digest(&self, to: &str, crates: &[CrateSummary]) -> Result<()>;
}

/// Type alias for dynamic mailer implementation.
pub type MailerTraitPtr = std::sync::Arc<dyn MailerTrait>;

/// Factory function to create a `MailerTrait` implementation.
///
/// Note: this is implemented in the `mail` infrastructure layer, and re-exported
/// here to maintain domain-layer abstraction.  The concrete type (`HtmlMailer`) is
/// deliberately hidden.
pub use crate::mail::create_mailer;
