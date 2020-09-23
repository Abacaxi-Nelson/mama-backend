use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub results: Vec<T>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonVal {
    pub success: bool
}