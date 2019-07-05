use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializableMessage {
    pub nickname: String,
    pub message: Option<String>,
    pub msg_type: Option<String>,
}

pub fn format_ws_message(
    nickname: &str,
    message: Option<String>,
    msg_type: Option<String>,
) -> String {
    let message = json!({
        "nickname": nickname,
        "message": message,
        "msg_type": msg_type
    });
    message.to_string()
}
