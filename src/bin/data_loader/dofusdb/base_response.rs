use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct BaseResponse {
    pub total: u32,
    pub limit: u32,
    pub skip: u32,
    pub data: Value,
}
