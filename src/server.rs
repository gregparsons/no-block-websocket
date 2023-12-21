//! server.rs

use std::{net::TcpListener};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;
use crossbeam_channel::{Receiver, RecvError};

use tungstenite::{Message, WebSocket};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;

const TEST_SHUTDOWN_TIMER_SEC:u64 = 30;

#[derive(Debug)]
pub enum Msg {
    Stop,
    StartPing,
}

#[derive(Debug)]
pub struct Server;
impl Server {

    fn control_comms(rx: Receiver<Msg>, arc: Arc<Mutex<WebSocket<TcpStream>>>, ) {

        loop {
            match rx.recv() {
                Ok(msg) => {

                    tracing::debug!("[client::control_comms] {msg:?}");
                    match msg {
                        Msg::Stop => {

                        },
                        Msg::StartPing => {

                        },
                    }


                }
                Err(e) => {
                    tracing::error!("[client][control_comms] {e:?}");
                }
            }
        }
    }

    pub fn run(s_rx: Receiver<Msg>) {

        let mut handles = vec![];

        let tcp_listener = TcpListener::bind("127.0.0.1:3012").unwrap();

        // for stream in listener.incoming()
        match tcp_listener.accept() {
            Ok((tcp_stream, _addr)) => {

                let tcp_stream = Arc::new(tcp_stream);

                match tungstenite::accept(tcp_stream.try_clone().unwrap()){
                    Ok(ws) => {

                        // prevent read() from blocking everything; has to be done after handshake
                        tcp_stream.set_nonblocking(true).unwrap();
                        let ws_arc:Arc<Mutex<WebSocket<TcpStream>>> = Arc::new(Mutex::new(ws));


                        // start the control panel
                        let ws0 = ws_arc.clone();
                        let h = spawn(move ||{
                            Server::control_comms(s_rx, ws0);
                        });
                        handles.push(h);



                        // TEST: shutdown after a while
                        let ws1 = ws_arc.clone();
                        let handle = spawn(move ||{
                            Server::shutdown(ws1);
                        });
                        handles.push(handle);

                        // Read loop
                        let ws2 = ws_arc.clone();
                        let handle = spawn(move || {
                            // read loop
                            loop {
                                {
                                    let mut ws2 = ws2.lock().unwrap();

                                    // read...
                                    match ws2.read() {
                                        Ok(msg) => {
                                            match msg {
                                                Message::Text(txt) => {
                                                    tracing::info!("[server] received text: {}", &txt);
                                                    match ws2.send(Message::Text(format!("server rcvd: {txt}"))) {
                                                        Ok(_) => {},
                                                        Err(e) => {
                                                            tracing::error!("[server] send after close: {e:?}");
                                                            // ws2.close(None).unwrap();
                                                            break;
                                                        }
                                                    }
                                                }
                                                Message::Ping(_ping) => {
                                                    tracing::info!("[server] rcvd: PING");
                                                    let _ = ws2.send(Message::Pong(vec![]));
                                                },
                                                // Message::Binary(Vec<u8>)=>{
                                                //
                                                // },
                                                // Message::Pong(Vec<u8>)=>{
                                                //
                                                // },
                                                // Message::Close(Option<CloseFrame<'static>>),
                                                // Message::Frame(Frame),
                                                _ => {}
                                            }
                                        }
                                        Err(_e) => {
                                            // tracing::error!("[server] read error {e:?}");
                                        }
                                    }
                                }
                                // mutex now unlocked

                                // tiny sleep to not over-poll; what's the right number? no clue, just need to not block the websocket w/read() in case we need to send something
                                std::thread::sleep(Duration::from_millis(1));
                            }
                        });
                        handles.push(handle);
                    }
                    Err(e) => {
                        // tungstenite accept
                        tracing::error!("[server] tungstenite accept error: {e:?}");
                    }
                }
            },
            Err(e) => {
                // tcp listener accept
                tracing::error!("[server] TcpStream accept error: {e:?}");
            }
        }

        for h in handles {
            h.join().unwrap();
        }

    }


    fn shutdown(ws1: Arc<Mutex<WebSocket<TcpStream>>>) {
        tracing::error!("[server] closing in {TEST_SHUTDOWN_TIMER_SEC} seconds");
        std::thread::sleep(Duration::from_secs(TEST_SHUTDOWN_TIMER_SEC));
        tracing::error!("[server] closing...");
        let mut ws1 = ws1.lock().unwrap();
        match ws1.close(Some(CloseFrame{ code: CloseCode::Normal, reason: Default::default() })) {
            Ok(_) => { tracing::debug!("[server] closed)"); },
            Err(e) => {tracing::debug!("[server] close error: {e:?})"); },
        }
    }

}


