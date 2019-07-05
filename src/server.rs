use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashSet;
use std::option::Option;
use std::rc::Rc;
use ws::{listen, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

#[derive(Serialize, Deserialize)]
struct SerializableMessage {
    nickname: String,
    message: Option<String>,
    msg_type: Option<String>,
}

type Users = Rc<RefCell<HashSet<String>>>;
// Basically store connections in a vec, then remove them when client disconnects
// This way we can try and stop echoing back the message you just sent to yourself
pub fn server() {
    let users = Users::new(RefCell::new(HashSet::with_capacity(10_000)));
    listen("127.0.0.1:3012", move |out| Server {
        out,
        connections: users.clone(),
    })
    .unwrap();
}

struct Server {
    connections: Users,
    out: Sender,
}

impl Server {
    fn add_connection(&mut self, username: String) {
        self.connections.borrow_mut().insert(username);
    }
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let text = msg.into_text().unwrap().clone();
        let parsed: SerializableMessage = serde_json::from_str(&text).unwrap();
        match parsed.msg_type.as_ref().map(|s| &s[..]) {
            Some("join_server") if self.connections.borrow().contains(&parsed.nickname) => {
                let message = json!({
                    "nickname": "server",
                    "message": Some("A user by that name already exists."),
                });
                self.out.send(message.to_string())
            }
            Some("join_server") => {
                self.add_connection(parsed.nickname);
                Ok(())
            }
            None => self.out.broadcast(text).map_err(|err| {
                Error::new(
                    ErrorKind::Internal,
                    format!("Unable to send message: {:?}.", err),
                )
            }),
            _ => Ok(()),
        }
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }
}
