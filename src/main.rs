//! main.rs
//!
//!

mod init;
mod client;
mod server;

fn main(){

    init::init("tungstenite");

    tracing::debug!("[main]");

    let h1 = std::thread::spawn(||{
        server::Server::run();
    });

    let h2 = std::thread::spawn(|| {
        let mut c = client::Client::new();
        c.run();
    });

    h1.join().unwrap();
    h2.join().unwrap();


}