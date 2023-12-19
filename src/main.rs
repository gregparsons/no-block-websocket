//! main.rs
//!
//!

use std::time::Duration;
use tokio::task::spawn_blocking;

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

        spawn_blocking(||{
            server::run();
        });

        spawn_blocking(||{
            let mut c = client::Client::new();
            c.run();

        });

    });


    // std::thread::spawn(||{
    //     server::run();
    // });

    // std::thread::sleep(Duration::from_secs(1));
    //
    // let mut c = client::Client::new();
    // c.run();


}