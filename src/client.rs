use ws::{connect, CloseCode};
use std::io;

pub fn client() {
   connect("ws://127.0.0.1:3012", |out| {
      println!("Starting chat...");

      let mut message = String::new();
      io::stdin().read_line(&mut message)
          .expect("Unable to read message");

      out.send(message);

      move |msg| {
         println!("Got message");
         out.close(CloseCode::Normal)
      }

   }).unwrap()
}