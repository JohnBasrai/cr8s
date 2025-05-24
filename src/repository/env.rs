//! Shared environment helpers for repository modules.

/// Reads an environment variable and parses it into a value, falling back to a default.
///
/// # Parameters
/// - `$name`: The name of the environment variable (string literal or expression)
/// - `$default`: A default value (any type that implements `Clone`)
///
#[macro_export]
macro_rules! get_env_with_default {
    ($ty:ty, $name:expr, $default:expr) => {{
        std::env::var($name)
            .map(|val| val.parse::<$ty>().unwrap_or($default))
            .unwrap_or($default)
    }};
}
