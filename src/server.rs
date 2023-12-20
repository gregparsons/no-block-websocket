//! server.rs

use std::{net::TcpListener};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::spawn;

use tungstenite::{accept_hdr, handshake::server::{Request, Response}, Message, WebSocket};

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
                let mut ws = accept_hdr(stream.unwrap(), callback).unwrap();

                loop {
                    let msg = ws.read().unwrap();

                    match msg {
                        Message::Text(txt) => {
                            tracing::info!("[server] received text: {}", &txt);
                            ws.send(Message::Text(format!("server rcvd: {txt}"))).unwrap();
                        }
                        Message::Ping(_ping) => {
                            // Vec<u8>
                            let _ = ws.send(Message::Pong(vec![]));
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
            });
        }
    }
}

