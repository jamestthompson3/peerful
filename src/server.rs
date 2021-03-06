use crate::shared;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use ws::{listen, CloseCode, Error, ErrorKind, Handler, Message, Result, Sender};

type Users = Rc<RefCell<HashSet<String>>>;
// Basically store connections in a vec, then remove them when client disconnects
// This way we can try and stop echoing back the message you just sent to yourself
pub fn server() {
    let users = Users::new(RefCell::new(HashSet::with_capacity(10_000)));
    listen("127.0.0.1:3012", move |out| Server {
        out,
        connections: users.clone(),
        user: None,
    })
    .unwrap();
}

struct Server {
    connections: Users,
    out: Sender,
    user: Option<String>,
}

impl Server {
    fn add_connection(&mut self, username: String) {
        self.connections.borrow_mut().insert(username);
    }
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let text = msg.into_text().unwrap().clone();
        let parsed: shared::SerializableMessage = serde_json::from_str(&text).unwrap();
        match parsed.msg_type.as_ref().map(|s| &s[..]) {
            Some("join_server") if self.connections.borrow().contains(&parsed.nickname) => {
                let message = shared::format_ws_message(
                    "server",
                    Some("A user by that name already exists.".to_string()),
                    Some("user_taken_error".to_string()),
                );
                self.out.send(message)
            }
            Some("join_server") => {
                let name = parsed.nickname.clone();
                self.add_connection(parsed.nickname);
                self.user = Some(name.clone());
                let message = shared::format_ws_message(
                    "server",
                    Some(format!("{} has joined", name).to_string()),
                    None,
                );
                self.out.broadcast(message)
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
        let user = self.user.clone();
        let message = shared::format_ws_message(
            "server",
            Some(format!("{} has left", &user.clone().unwrap())),
            None,
        );
        self.out.broadcast(message).unwrap();
        self.connections.borrow_mut().remove(&user.unwrap());
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }
}
