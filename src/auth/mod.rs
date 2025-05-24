//! Auth module exports default crypto implementation.

mod password;

pub use password::create_password_hasher;

#[cfg(test)]
mod mock_password;

#[cfg(test)]
pub use mock_password::MockPasswordHasher;

#[cfg(test)]
mod tests;
