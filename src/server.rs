use std::cell::RefCell;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::rc::Rc;
use ws::{listen, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Result, Sender};

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
    fn add_connection(&mut self, address: Option<SocketAddr>) {
        let addr = address.unwrap();
        self.connections.borrow_mut().insert(format!("{}", addr));
    }
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Server got message {}", msg);
        self.out.broadcast(msg).map_err(|err| {
            Error::new(
                ErrorKind::Internal,
                format!("Unable to send message: {:?}.", err),
            )
        })
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }

    fn on_open(&mut self, handshake: Handshake) -> Result<()> {
        println!("peer-address: {:?}", handshake.peer_addr);
        self.add_connection(handshake.peer_addr);
        println!("Connections: {:?}", self.connections);
        Ok(())
    }
}
