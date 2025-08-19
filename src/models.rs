use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingRequest {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub username: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,

    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub region: Option<String>,
}