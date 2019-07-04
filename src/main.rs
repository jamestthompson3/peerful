mod client;
mod server;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        client::client();
    }
    server::server();
}
