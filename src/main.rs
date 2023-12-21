//! main.rs
//!
//!

use crate::server::Msg;

mod init;
mod client;
mod server;

fn main(){

    init::init("tungstenite");
    tracing::debug!("[main]");


    let (s_tx, s_rx) = crossbeam_channel::unbounded::<Msg>();

    let h1 = std::thread::spawn(||{
        server::Server::run(s_rx);
    });

    let (client_tx, client_rx) = crossbeam_channel::unbounded::<Msg>();
    let h2 = std::thread::spawn(|| {
        let mut c = client::Client::new(client_rx);
        c.run();
    });

    client_tx.send(Msg::StartPing).unwrap();


    h1.join().unwrap();
    h2.join().unwrap();


}