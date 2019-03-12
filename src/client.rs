use std::io;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::{connect, Handler, CloseCode, Sender, Error, ErrorKind, Handshake, Message};

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
    let client = thread::spawn(move || {
        connect("ws://127.0.0.1:3012", |sender| SocketClient {
            ws_sender: sender,
            thread: tx.clone(),
        })
        .unwrap();
    });

    if let Ok(Event::Connect(sender)) = rx.recv() {
        // Main loop
        loop {
            println!("Enter your message:");
            // Get user input
            let mut message = String::new();
            io::stdin()
                .read_line(&mut message)
                .expect("Unable to read message");

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
       println!("Received {}", msg);
       Ok(())
   }
}

