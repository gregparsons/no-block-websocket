//! client.rs

use std::net::TcpStream;
use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;


pub struct Client{
    socket: WebSocket<MaybeTlsStream<TcpStream>>
}

impl Client {

    pub fn new()->Self{
        let (mut socket, response) =
            connect(url::Url::parse("ws://localhost:3012/socket")
                .unwrap())
                .expect("Can't connect");

        tracing::debug!("[client] Connected to the server");
        tracing::debug!("[client] Response HTTP code: {}", response.status());
        tracing::debug!("[client] Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            tracing::debug!("[client] * {}", header);
        }

        Client{ socket }
    }

    pub fn run(&mut self) {
        self.socket.send(Message::Text("Hello WebSocket".into())).unwrap();

        loop {

            let msg:Message = self.socket.read().expect("[client] Error reading message");

            tracing::debug!("[client] Received: {}", msg);


        }
        // socket.close(None);
    }
}