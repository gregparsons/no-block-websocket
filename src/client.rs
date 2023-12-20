//! client.rs

use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;


pub struct Client{
    socket_arc: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>
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

        Client{ socket_arc: Arc::new(Mutex::new(socket)) }
    }

    pub fn run(&mut self) {


        let s = self.socket_arc.clone();
        let h1 = std::thread::spawn(move || {
            for i in 0..10{
                tracing::debug!("[client] spawned ping thread: {i}");
                {
                    let mut unlocked_socket = s.lock().expect("[client] ping loop couldn't unlock");
                    // unlocked_socket.send(Message::Ping(vec![])).expect("[Client] ping send failed");
                    unlocked_socket.send(Message::Text(format!("hello {}", i))).expect("[Client] ping send failed");
                    std::thread::sleep(Duration::from_secs(1));
                }
            }

        });

        tracing::debug!("[client] moving on after spawned ping thread");

        let s2 = self.socket_arc.clone();
        let h2 = std::thread::spawn(move ||{

            tracing::debug!("[client] 2nd spawn_blocking...");

            {
                let mut unlocked_socket = s2.lock().unwrap();
                unlocked_socket.send(Message::Text("Hello WebSocket".into())).unwrap();
            }
            loop {
                tracing::debug!("[client] second loop...");
                let mut unlocked_socket = s2.lock().unwrap();

                let msg:Message = unlocked_socket.read().expect("[client] Error reading message");
                tracing::debug!("[client] Received: {}", msg);
            }

        });
        //
        // // join!(h, h2); // expect("[client] join fail");
        //
        // h1.join().unwrap();
        // h2.join().unwrap();

        loop {
            tracing::debug!("[client] loop to prevent closing websocket");
            std::thread::sleep(Duration::from_secs(1));
        }

        // socket.close(None);

    }
}