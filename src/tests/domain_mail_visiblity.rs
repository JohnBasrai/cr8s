// tests/domain_mail_visibility.rs

use cr8s::domain::{create_mailer, MailerTrait, MailerTraitPtr}; // ✅ Should compile

#[tokio::test]
async fn mailer_factory_and_trait_are_visible() {
    // We don’t call create_mailer() here because we don’t have a DB connection
    // We’re only verifying trait and factory function are publicly visible and usable
    let _trait_ptr: Option<MailerTraitPtr> = None;
    let _factory_fn = create_mailer;
}

// 🔥 This block should fail to compile if uncommented — confirm HtmlMailer is NOT accessible
/*
use cr8s::mail::html_mailer::HtmlMailer;

#[test]
fn should_not_see_concrete_mailer() {
    let _ = HtmlMailer {}; // This must fail — HtmlMailer is not public
}
*/
