use std::io;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::{connect, Error, ErrorKind, Handler, Handshake, Message, Sender};

enum Event {
    Connect(Sender),
    Disconnect,
}

struct SocketClient {
    ws_sender: Sender,
    thread: TSender<Event>,
}

pub fn client() {
    let (tx, rx) = channel();

    // Run client thread with channel to give it's WebSocket message sender back to us
    thread::spawn(move || {
        connect("ws://127.0.0.1:3012", |sender| SocketClient {
            ws_sender: sender,
            thread: tx.clone(),
        })
        .unwrap();
    });

    if let Ok(Event::Connect(sender)) = rx.recv() {
        // Main loop
        display(&"Enter message:");
        loop {
            let mut message = String::new();
            io::stdin()
                .read_line(&mut message)
                .expect("Unable to read message");

            if let Ok(Event::Disconnect) = rx.try_recv() {
                break;
            }
            display(&format!("{:?}", sender.token()));
            sender.send(message).unwrap();
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
        display(&format!(">>> {}", msg.into_text().unwrap()));
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
