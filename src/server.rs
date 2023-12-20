//! server.rs

use std::{net::TcpListener};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;

use tungstenite::{accept_hdr, handshake::server::{Request, Response}, Message, WebSocket};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;

#[derive(Debug)]
pub struct Server{
    // socket_arc_opt:Arc<Mutex<Option<WebSocket<TcpStream>>>>,
}

impl Server {
    pub fn run() {

        let server = TcpListener::bind("127.0.0.1:3012").unwrap();
        for stream in server.incoming() {
            spawn(move || {
                let callback = |req: &Request, mut response: Response| {
                    println!("Received a new ws handshake");
                    println!("The request's path is: {}", req.uri().path());
                    println!("The request's headers are:");
                    for (ref header, _value) in req.headers() {
                        println!("* {}", header);
                    }

                    // Let's add an additional header to our response to the client.
                    let headers = response.headers_mut();
                    headers.append("MyCustomHeader", ":)".parse().unwrap());
                    headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

                    Ok(response)
                };
                let ws = accept_hdr(stream.unwrap(), callback).unwrap();

                let ws_arc:Arc<Mutex<WebSocket<TcpStream>>> = Arc::new(Mutex::new(ws));


                let ws1 = ws_arc.clone();
                spawn(move ||{
                    std::thread::sleep(Duration::from_secs(20));
                    tracing::error!("[server] closing...");
                    let mut ws1 = ws1.lock().unwrap();
                    ws1.close(Some(CloseFrame{ code: CloseCode::Normal, reason: Default::default() })).unwrap();

    
                });

                let ws2 = ws_arc.clone();
                loop {

                    let mut ws2 = ws2.lock().unwrap();

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
            });
        }
    }
}

