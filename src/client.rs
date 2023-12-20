//! client.rs

use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tungstenite::{connect, Message, WebSocket};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::stream::MaybeTlsStream;


pub struct Client{
    socket_arc: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>
}

impl Client {

    pub fn new()->Self{
        let (socket, response) =
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


        let s0 = self.socket_arc.clone();


        let _ = std::thread::spawn(move || {
            for i in 0..10{
                tracing::debug!("[client] spawned ping thread: {i}");
                {
                    let mut unlocked_socket = s0.lock().expect("[client] ping loop couldn't unlock");
                    // unlocked_socket.send(Message::Ping(vec![])).expect("[Client] ping send failed");
                    match unlocked_socket.send(Message::Text(format!("hello {}", i))) {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!("[client] send error: {:?}", e);
                        }
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            }

        });

        tracing::debug!("[client] moving on after spawned ping thread");

        let s1 = self.socket_arc.clone();
        let _ = std::thread::spawn(move ||{

            tracing::debug!("[client] 2nd spawn_blocking...");

            {
                let mut unlocked_socket = s1.lock().unwrap();
                unlocked_socket.send(Message::Text("Hello WebSocket".into())).unwrap();
            }
            loop {

                tracing::debug!("[client] read loop...");
                let s3 = s1.clone();
                {
                    let mut ws2 = s3.lock().unwrap();
                    let msg: Message = ws2.read().expect("[client] Error reading message");

                    match msg {

                        Message::Text(txt) => {
                            tracing::info!("[client::text] rcvd: {}", &txt);
                            // ws2.send(Message::Text(format!("server rcvd: {txt}"))).unwrap();
                        }
                        Message::Ping(_) => {
                            tracing::debug!("[client] rcvd: PING");
                            let _ = ws2.send(Message::Pong(vec![]));
                        },
                        Message::Pong(_) => {
                            tracing::debug!("[client] rcvd: PONG");
                        },
                        // Message::Binary(Vec<u8>)=>{
                        //
                        // },
                        // Message::Pong(Vec<u8>)=>{
                        //
                        // },
                        Message::Close(_)=> {
                            tracing::error!("[client] received Message::Close");
                            break;
                            // ws2.close(None).unwrap();
                        },
                        // Message::Frame(Frame),
                        _ => {}
                    }
                }
            }
        });


        // timeout
        let s4 = self.socket_arc.clone();
        let h3 = std::thread::spawn(move ||{

            tracing::debug!("[client] closing websocket in 20 seconds");
            std::thread::sleep(Duration::from_secs(15));
            let mut unlocked_socket = s4.lock().unwrap();
            tracing::debug!("[client] closing client websocket");
            unlocked_socket.close(Some(CloseFrame{ code: CloseCode::Normal, reason: Default::default() })).unwrap();

        });

        h3.join().unwrap();

        // socket.close(None);

    }
}