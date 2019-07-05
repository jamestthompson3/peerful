use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use std::option::Option;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::{connect, Error, ErrorKind, Handler, Handshake, Message, Sender};

#[derive(Serialize, Deserialize)]
struct SerializableMessage {
    nickname: String,
    message: Option<String>,
    msg_type: Option<String>,
}

enum Event {
    Connect(Sender),
    Disconnect,
}

struct SocketClient {
    ws_sender: Sender,
    thread: TSender<Event>,
}

pub fn client() {
    let (chan_send, chan_recv) = channel();

    display(&"Enter nickname:");
    let mut nickname = String::new();
    io::stdin()
        .read_line(&mut nickname)
        .expect("Unable to read nickname");
    // Run client thread with channel to give it's WebSocket message sender back to us
    thread::spawn(move || {
        connect("ws://127.0.0.1:3012", |sender| SocketClient {
            ws_sender: sender,
            thread: chan_send.clone(),
        })
        .unwrap();
    });

    if let Ok(Event::Connect(sender)) = chan_recv.recv() {
        let mut name_lines = nickname.lines();
        let message = json!({
            "nickname": name_lines.next().unwrap(),
            "msg_type": Some("join_server")
        });
        sender.send(message.to_string()).unwrap();
        display(&"Enter message:");
        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Unable to read input");

            if let Ok(Event::Disconnect) = chan_recv.try_recv() {
                break;
            }
            let mut name_lines = nickname.lines();
            let message = json!({
                "message": Some(input),
                "nickname": name_lines.next().unwrap(),

            });
            sender.send(message.to_string()).unwrap();
        }
    }
}

impl Handler for SocketClient {
    fn on_open(&mut self, _: Handshake) -> Result<(), Error> {
        self.thread
            .send(Event::Connect(self.ws_sender.clone()))
            .map_err(|err| {
                Error::new(
                    ErrorKind::Internal,
                    format!("Unable to communicate between threads: {:?}.", err),
                )
            })
    }

    fn on_message(&mut self, msg: Message) -> Result<(), Error> {
        let text = msg.into_text().unwrap().clone();
        let parsed: SerializableMessage = serde_json::from_str(&text).unwrap();
        // TODO set as env vars for customization
        let display_name = format!("{}", parsed.nickname).bright_cyan();
        let separator = ">>>".bright_black();
        display(&format!(
            "{} {} {}",
            display_name,
            separator,
            parsed.message.unwrap()
        ));
        display(&"Enter message:");
        Ok(())
    }
}

fn display(string: &str) {
    let mut msg = term::stdout().unwrap();
    msg.carriage_return().unwrap();
    msg.delete_line().unwrap();
    println!("{}", string);
}
