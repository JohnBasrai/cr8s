/// Ensure a reqwest response matches the expected status code.
///
/// # Usage
/// ```
/// ensure_status!(response, StatusCode::OK);
/// ensure_status!(response, StatusCode::CREATED, "crate creation failed");
/// ```
///
/// If the third argument is given, it prefixes the error message.
#[macro_export]
macro_rules! ensure_status {
    // with context prefix
    ($response:expr, $expected:expr, $context:expr) => {{
        let status = $response.status();
        ::anyhow::ensure!(
            status == $expected,
            "{}: expected {}, got {}",
            $context,
            $expected,
            status
        );
    }};
    // without context prefix
    ($response:expr, $expected:expr) => {{
        let status = $response.status();
        ::anyhow::ensure!(
            status == $expected,
            "expected {}, got {}",
            $expected,
            status
        );
    }};
}
