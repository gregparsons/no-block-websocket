//! server.rs

use std::{net::TcpListener};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;

use tungstenite::{Message, WebSocket};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;

#[derive(Debug)]
pub struct Server;
impl Server {
    pub fn run() {
        let tcp_listener = TcpListener::bind("127.0.0.1:3012").unwrap();

        // for stream in listener.incoming()

        match tcp_listener.accept() {
            Ok((tcp_stream, _addr)) => {
                match tungstenite::accept(tcp_stream){
                    Ok(ws) => {
                        let ws_arc:Arc<Mutex<WebSocket<TcpStream>>> = Arc::new(Mutex::new(ws));


                        // closer thread
                        let ws1 = ws_arc.clone();
                        spawn(move ||{
                            let closing_time = 30;
                            tracing::error!("[server] closing in {closing_time} seconds");
                            std::thread::sleep(Duration::from_secs(closing_time));
                            tracing::error!("[server] closing...");
                            let mut ws1 = ws1.lock().unwrap();
                            match ws1.close(Some(CloseFrame{ code: CloseCode::Normal, reason: Default::default() })) {
                                Ok(_) => { tracing::debug!("[server] closed)"); },
                                Err(e) => {tracing::debug!("[server] close error: {e:?})"); },
                            }
                        });

                        let ws2 = ws_arc.clone();
                        spawn(move || {

                            // read loop
                            loop {

                                {
                                    let mut ws2 = ws2.lock().unwrap();

                                    // read loop
                                    if let Ok(msg) = ws2.read() {
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
                                                // Vec<u8>
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
                                }
                                // mutex now unlocked
                            }
                        });


                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }
}

