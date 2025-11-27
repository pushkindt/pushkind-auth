//! Authentication-related DTOs.

use serde::{Deserialize, Serialize};

/// DTO carrying an issued session token.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionTokenDto {
    pub token: String,
}

impl From<String> for SessionTokenDto {
    fn from(token: String) -> Self {
        Self { token }
    }
}
