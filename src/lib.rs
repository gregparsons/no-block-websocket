//! lib.rs
//!
//!

pub mod init;
pub mod client;
pub mod server;
pub mod command;

pub const TEST_SHUTDOWN_TIMER_SEC:u64 = 1;
pub const SERVER_READ_POLL_MS:u64 = 1;
pub const CLIENT_READ_POLL_MS:u64 = 1;

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::{client, init, server};
    use crate::command::Cmd;

    /// client PINGs server once per second; server replies with PONG
    /// Client gets told to shutdown after four seconds
    #[test]
    fn simulated_main() {
        init::init("no-block-websocket");
        tracing::debug!("[main]");


        let (_server_tx, server_rx) = crossbeam_channel::unbounded::<Cmd>();

        let h1 = std::thread::spawn(|| {
            server::Server::run(server_rx);
        });

        let (client_tx, client_rx) = crossbeam_channel::unbounded::<Cmd>();
        let h2 = std::thread::spawn(|| {
            let mut c = client::Client::new(client_rx);
            c.run();
        });
        
        // test
        client_tx.send(Cmd::StartPing).unwrap();
        std::thread::sleep(Duration::from_secs(4));
        client_tx.send(Cmd::Shutdown).unwrap();

        h1.join().unwrap();
        h2.join().unwrap();
    }
}