//! Network-independent, synthetic-only Fame foundations.
//!
//! This crate performs no I/O and contains no Instagram, PIR, or mixnet client.

pub mod agent;
pub mod corpus;
pub mod fame;
pub mod protocol;

pub mod identity {
    pub fn normalize(username: &str) -> String {
        username.trim().to_lowercase()
    }

    pub fn is_valid_username(username: &str) -> bool {
        let username = username.trim();
        !username.is_empty()
            && username.len() <= 30
            && username
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
    }
}
