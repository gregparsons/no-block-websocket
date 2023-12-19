//! main.rs
//!
//!

use std::time::Duration;

mod init;

fn main(){
    init::init("rust_basic_template_2023");
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

        let my_closure = async  {
            for i in 0..5{
                tracing::debug!("[loop] {i}");
                tokio::time::sleep(Duration::from_secs(1)).await;

            }
        };

        tokio::join!(my_closure);

    });
}