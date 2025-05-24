//! Mail module exports the default mail delivery implementation.

mod html_mailer;
pub use html_mailer::create_mailer;
pub use html_mailer::HtmlMailer as Mailer;
