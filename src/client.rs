use std::io;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::{connect, Handler, CloseCode, Sender};

enum Event {
    Connect(Sender),
    Disconnect,
}

struct SocketClient {
    ws_sender: Sender,
    thread: TSender<Event>,
}

impl Handler for SocketClient {
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
            // Get user input
            let mut message = String::new();
            io::stdin()
                .read_line(&mut message)
                .expect("Unable to read message");

            sender.send(message);

            // move |msg| {
            //     println!("Got message");
            //     sender.close(CloseCode::Normal)
            // }
        }
    }
}
