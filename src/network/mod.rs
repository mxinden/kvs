use serde::{Deserialize, Serialize};

/// Request send by the client.
#[derive(Serialize, Deserialize, Debug)]
pub enum Req {
    /// Get value for given key.
    Get(String),
    /// Set value for given key.
    Set(String, String),
    /// Remove value for given key.
    Remove(String),
}

/// Response send by server.
pub type Resp = Result<SuccResp, Error>;

/// Success response send by the server.
#[derive(Serialize, Deserialize, Debug)]
pub enum SuccResp {
    /// Successful get response containing value.
    Get(Option<String>),
    /// Successful set response.
    Set,
    /// Successful remove response.
    Remove,
}

/// Failure response send by server.
#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    /// Error send by server.
    Server(String)
}
