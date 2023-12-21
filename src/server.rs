//! server.rs

use std::{net::TcpListener};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;
use crossbeam_channel::{Receiver};
use tungstenite::{Message, WebSocket};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use crate::command::Cmd;
use crate::{SERVER_READ_POLL_MS, TEST_SHUTDOWN_TIMER_SEC};

#[derive(Debug)]
pub struct Server;
impl Server {

    fn control_comms(rx: Receiver<Cmd>, ws_arc: Arc<Mutex<WebSocket<TcpStream>>>, ) {

        loop {
            match rx.recv() {
                Ok(msg) => {

                    tracing::debug!("[client::control_comms] {msg:?}");
                    match msg {
                        Cmd::Shutdown => {
                            Server::shutdown(ws_arc.clone());
                            break;
                        }
                        _ => {},
                    }
                }
                Err(e) => {
                    tracing::error!("[client][control_comms] {e:?}");
                }
            }
        }
    }

    pub fn run(s_rx: Receiver<Cmd>) {

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

                        // command/control thread may want to write so need to get a lock
                        let ws_arc:Arc<Mutex<WebSocket<TcpStream>>> = Arc::new(Mutex::new(ws));

                        // listen for control commands (inter-thread)
                        let ws0 = ws_arc.clone();
                        let h = spawn(move ||{ Server::control_comms(s_rx, ws0); });
                        handles.push(h);

                        // TEST: confirm read loop can be interrupted; shutdown after a while
                        // let ws1 = ws_arc.clone();
                        // let handle = spawn(move ||{
                        //     Server::shutdown(ws1);
                        // });
                        // handles.push(handle);

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
                                                            break;
                                                        }
                                                    }
                                                }
                                                Message::Ping(_ping) => {
                                                    tracing::info!("[server] rcvd: PING");
                                                    let _ = ws2.send(Message::Pong(vec![]));
                                                },
                                                // Message::Binary(Vec<u8>)=>{},
                                                // Message::Pong(Vec<u8>)=>{},
                                                // Message::Close(Option<CloseFrame<'static>>),
                                                // Message::Frame(Frame),
                                                _ => {}
                                            }
                                        }
                                        Err(_e) => {}, // tracing::error!("[server] read error {e:?}");
                                    }
                                }
                                // mutex lock released

                                // tiny sleep; what's the right number? no clue, just need to not block the websocket w/read() in case we need to send something
                                std::thread::sleep(Duration::from_millis(SERVER_READ_POLL_MS));
                            }
                        });
                        handles.push(handle);
                    }
                    Err(e) => tracing::error!("[server] tungstenite accept error: {e:?}"),
                }
            },
            Err(e) => tracing::error!("[server] TcpStream accept error: {e:?}"),
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


