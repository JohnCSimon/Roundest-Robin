use rand::Rng;

use super::Endpoint;

#[async_trait::async_trait]
pub trait EndpointStore {
    async fn add_endpoint(&mut self, endpoint: Endpoint) -> Result<(), EndpointStoreError>;
    async fn get_next_endpoint(&self) -> Result<Endpoint, EndpointStoreError>;
    async fn get_all_endpoints(&self) -> Result<Vec<Endpoint>, EndpointStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum EndpointStoreError {
    EndpointAlreadyExists,
    NoEndpoints,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        let parsed_id =
            uuid::Uuid::parse_str(&id).map_err(|_| "Invalid login attempt id".to_owned())?;
        Ok(Self(parsed_id.to_string()))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        let code_as_u32 = code
            .parse::<u32>()
            .map_err(|_| "Invalid 2FA code".to_owned())?;

        if (100_000..=999_999).contains(&code_as_u32) {
            Ok(Self(code))
        } else {
            Err("Invalid 2FA code".to_owned())
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        Self(rand::thread_rng().gen_range(100_000..=999_999).to_string())
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
