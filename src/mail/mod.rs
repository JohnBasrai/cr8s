//! Email delivery interface and implementation for cr8s.
//!
//! Provides a `MailerTrait` abstraction used by both CLI and server.
//! The default implementation sends HTML email using Tera templates
//! and logs delivery attempts for observability. Mail module exports
//! the default mail delivery implementation.

mod html_mailer;
pub use html_mailer::create_mailer;
pub use html_mailer::HtmlMailer as Mailer;
