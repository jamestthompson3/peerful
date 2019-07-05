use crate::shared;
use colored::*;
use std::io;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::{connect, Error, ErrorKind, Handler, Handshake, Message, Sender};

enum Event {
    Connect(Sender, String),
    Disconnect,
}

struct SocketClient {
    ws_sender: Sender,
    thread: TSender<Event>,
    nickname: String,
    loop_init: bool,
}

pub fn client() {
    let (chan_send, chan_recv) = channel();

    // Run client thread with channel to give it's WebSocket message sender back to us
    thread::spawn(move || {
        connect("ws://127.0.0.1:3012", |sender| SocketClient {
            ws_sender: sender,
            thread: chan_send.clone(),
            nickname: String::new(),
            loop_init: false,
        })
        .unwrap();
    });

    if let Ok(Event::Connect(sender, nickname)) = chan_recv.recv() {
        display(&"Enter message: ");
        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Unable to read input");

            if let Ok(Event::Disconnect) = chan_recv.try_recv() {
                break;
            }
            let mut name_lines = nickname.lines();
            let message = shared::format_ws_message(
                name_lines.next().unwrap(),
                Some(input.to_string()),
                None,
            );
            sender.send(message).unwrap();
        }
    }
}

impl Handler for SocketClient {
    fn on_open(&mut self, _: Handshake) -> Result<(), Error> {
        display(&"Enter nickname:");
        io::stdin()
            .read_line(&mut self.nickname)
            .expect("Unable to read nickname");
        let mut name_lines = self.nickname.lines();
        let msg_name = name_lines.next().unwrap();
        let message = shared::format_ws_message(msg_name, None, Some("join_server".to_string()));
        self.ws_sender.send(message)
    }

    fn on_message(&mut self, msg: Message) -> Result<(), Error> {
        let text = msg.into_text().unwrap().clone();
        let mut parsed: shared::SerializableMessage = serde_json::from_str(&text).unwrap();
        let is_error = get_msg_err(&mut parsed);
        display_message(&mut parsed, is_error, self.loop_init);
        match &parsed.msg_type.as_ref().map(|s| &s[..]) {
            Some("user_taken_error") => {
                self.nickname = String::new();
                display(&"Enter nickname:");
                io::stdin()
                    .read_line(&mut self.nickname)
                    .expect("Unable to read nickname");
                let mut name_lines = self.nickname.lines();
                let msg_name = name_lines.next().unwrap();
                let message =
                    shared::format_ws_message(msg_name, None, Some("join_server".to_string()));
                self.ws_sender.send(message).unwrap();
            }
            None if !self.loop_init => {
                let name = self.nickname.clone();
                self.thread
                    .send(Event::Connect(self.ws_sender.clone(), name))
                    .map_err(|err| {
                        Error::new(
                            ErrorKind::Internal,
                            format!("Unable to communicate between threads: {:?}.", err),
                        )
                    })
                    .unwrap();
                self.loop_init = true;
            }
            _ => {}
        }
        Ok(())
    }
}

fn get_msg_err<'a>(parsed: &'a mut shared::SerializableMessage) -> bool {
    return parsed.msg_type.is_some() && parsed.msg_type.as_mut().unwrap().contains("error");
}

fn display(string: &str) {
    let mut msg = term::stdout().unwrap();
    msg.carriage_return().unwrap();
    msg.delete_line().unwrap();
    println!("{}", string);
    msg.carriage_return().unwrap();
}

fn display_message<'a>(
    parsed: &'a mut shared::SerializableMessage,
    is_error: bool,
    loop_init: bool,
) {
    // TODO set as env vars for customization
    let display_name = if parsed.nickname == "server" {
        format!("{}", parsed.nickname).bright_magenta()
    } else {
        format!("{}", parsed.nickname).bright_cyan()
    };
    let separator = ">>>".bright_black();
    let msg_color = if is_error { "red" } else { "white" };
    let display_msg = parsed.message.as_mut().unwrap().color(msg_color);
    display(&format!("{} {} {}", display_name, separator, display_msg));
    if !is_error && loop_init {
        display(&"Enter message:");
    }
}
