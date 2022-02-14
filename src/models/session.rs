use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Auth token encoded as base-64
    pub auth: BearerToken,
    /// Expiration timestamp encoded with RFC 3339
    pub expires: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousSession {
    /// Expiration timestamp encoded with RFC 3339/ISO 8061
    pub expires: Timestamp,
}
