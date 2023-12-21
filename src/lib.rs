//! lib
//!
//!

use std::time::Duration;
use crate::command::Cmd;

mod init;
mod client;
mod server;
mod command;

const TEST_SHUTDOWN_TIMER_SEC:u64 = 1;
const SERVER_READ_POLL_MS:u64 = 1;
const CLIENT_READ_POLL_MS:u64 = 1;

mod tests {
    use std::time::Duration;
    use crate::{client, init, server};
    use crate::command::Cmd;

    /// client PINGs server once per second; server replies with PONG
    /// Client gets told to shutdown after four seconds
    #[test]
    fn simulated_main() {
        init::init("tungstenite");
        tracing::debug!("[main]");


        let (s_tx, s_rx) = crossbeam_channel::unbounded::<Cmd>();

        let h1 = std::thread::spawn(|| {
            server::Server::run(s_rx);
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