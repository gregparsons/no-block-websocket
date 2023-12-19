//! server.rs

use std::{net::TcpListener, thread::spawn};

use tungstenite::{accept_hdr, handshake::server::{Request, Response}, Message};

pub fn run() {
    println!("[server::run]");
    tracing::debug!("[run]");

    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, mut response: Response| {
                tracing::debug!("[server] Received a new ws handshake");
                tracing::debug!("[server] The request's path is: {}", req.uri().path());
                tracing::debug!("[server] The request's headers are:");
                for (ref header, _value) in req.headers() {
                    tracing::debug!("[server] * {}", header);
                }

                // Let's add an additional header to our response to the client.
                let headers = response.headers_mut();
                headers.append("MyCustomHeader", ":)".parse().unwrap());
                headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

                Ok(response)
            };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            loop {
                let msg = websocket.read().unwrap();

                match msg{

                    Message::Text(txt)=>{
                        tracing::debug!("[server] received text: {}", &txt);
                        websocket.send(Message::Text(txt)).unwrap();
                    }
                    // Message::Binary(Vec<u8>)=>{
                    //
                    // },
                    Message::Ping(ping)=>{
                        // Vec<u8>
                        let _ = websocket.send(Message::Pong(vec![]));

                    },
                    // Message::Pong(Vec<u8>)=>{
                    //
                    // },
                    // Message::Close(Option<CloseFrame<'static>>),
                    // Message::Frame(Frame),
                    _ =>{

                    }

                }


                // if msg.is_binary() || msg.is_text() {
                //     if msg.is_text(){
                //         tracing::debug!("[server] received: {:?}", msg)
                //     }
                //
                // }
            }
        });
    }
}