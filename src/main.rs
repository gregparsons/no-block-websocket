//! main.rs
//!
//!

use tokio::{try_join};
use tokio::task::{JoinHandle, spawn_blocking};

mod init;
mod client;
mod server;

fn main(){

    init::init("tungstenite");

    tracing::debug!("[main]");

    let tokio_runtime = tokio::runtime::Builder::
    // new_current_thread()
    new_multi_thread()
        .worker_threads(2)
        .on_thread_start(|| {})
        .on_thread_stop(|| {})
        .thread_name("actix")
        .enable_all()
        .build()
        .expect("Tokio runtime didn't start");
    tokio_runtime.block_on( async {

        let h1:JoinHandle<()> = spawn_blocking(||{
            server::run();
        });

        let h2:JoinHandle<()> = spawn_blocking(||{
            let mut c = client::Client::new();
            c.run();

        });

        // waits for all branches to complete
        let _ = try_join!(h1, h2);

    });



}