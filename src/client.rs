//! client.rs
//!
//! non-blocking websocket read() effectively polls instead of waiting and blocking the entire thread/socket

use std::net::{TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;
use crossbeam_channel::Receiver;
use tungstenite::{Message, WebSocket};
use tungstenite::client::client_with_config;
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::stream::MaybeTlsStream;
use crate::command::Cmd;
use crate::{CLIENT_READ_POLL_MS, TEST_SHUTDOWN_TIMER_SEC};


pub struct Client{
    socket_arc: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>
}

impl Client {

    fn control_comms(rx: Receiver<Cmd>, ws_arc: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) {

        let mut handles = vec![];

        loop {
            match rx.recv() {
                Ok(msg) => {

                    tracing::debug!("[client::control_comms] {msg:?}");
                    match msg {
                        Cmd::Shutdown => {
                            Client::shutdown(ws_arc.clone());
                            break;
                        }
                        Cmd::StartPing => {
                            // thread to send 100 pings
                            let s0 = ws_arc.clone();
                            let join_handle_0 = std::thread::spawn(move || {
                                Client::ping(s0);
                            });
                            handles.push(join_handle_0);
                            tracing::debug!("[client] spawned counter thread");
                        },
                    }
                }
                Err(e) => {
                    tracing::error!("[client][control_comms] {e:?}");
                }
            }
        }

        for h in handles {
            h.join().unwrap();
        }
    }


    pub fn new(client_rx: Receiver<Cmd>) ->Self{

        let tcp_stream = Arc::new(TcpStream::connect("127.0.0.1:3012").unwrap());
        let t1 = tcp_stream.clone();

        let (socket, response) = client_with_config(
            url::Url::parse("ws://localhost:3012/socket").unwrap(),
            MaybeTlsStream::Plain(t1.try_clone().unwrap()),
            None
        ).unwrap();
        // let (socket, response) = connect(url::Url::parse("ws://localhost:3012/socket").unwrap()).expect("Can't connect");

        // set non-blocking after handshake...it matters
        tcp_stream.set_nonblocking(true).expect("set_nonblocking call failed");

        tracing::debug!("[client] Connected to the server");
        tracing::debug!("[client] Response HTTP code: {}", response.status());
        tracing::debug!("[client] Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            tracing::debug!("[client] * {}", header);
        }


        let socket_arc = Arc::new(Mutex::new(socket));

        // start the control panel
        let ws0 = socket_arc.clone();
        let _h = spawn(move ||{
            Client::control_comms(client_rx, ws0);
        });

        Client{ socket_arc  }
    }


    /// PING


    pub fn run(&mut self) {

        let mut handles = vec![];






        // thread to read from the socket
        // read currently blocks, blocking the above "heartbeat"
        // https://github.com/snapview/tungstenite-rs/issues/11
        let s1 = self.socket_arc.clone();
        let join_handle_1 = std::thread::spawn(move ||{
            tracing::debug!("[client] starting read loop thread...");

            loop {

                // tracing::debug!("[client] reading...");
                let s3 = s1.clone();
                {
                    let mut ws2 = s3.lock().unwrap();

                    // https://www.reddit.com/r/rust/comments/dktiwf/reading_from_a_tcpstream_without_blocking/?rdt=54487
                    if ws2.can_read() {

                        match ws2.read() {
                            Ok(msg) => {

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
                                    Message::Close(_) => {
                                        tracing::error!("[client] received Message::Close");
                                        break;
                                    },
                                    _ => {}
                                }
                            }
                            Err(_e) => {
                                // todo: need to differentiate between errors when closed and nothing to read
                                // tracing::error!("[client] read error: {e:?}");
                            }
                        }
                    }
                }
                // read poll rate
                std::thread::sleep(Duration::from_millis(CLIENT_READ_POLL_MS));
            }
        });
        handles.push(join_handle_1);

        // TEST: close after a certain time
        // let s4 = self.socket_arc.clone();
        // let h3 = std::thread::spawn(move ||{
        //     Client::shutdown(s4);
        // });
        // handles.push(h3);

        for h in handles {
            h.join().unwrap();
        }

        // socket.close(None);


    }

    /// Both shutdown and ping are started externally with crossbeam "command" messages
    fn shutdown(s4: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) {
        tracing::error!("[client] closing client websocket in {TEST_SHUTDOWN_TIMER_SEC} seconds");
        std::thread::sleep(Duration::from_secs(TEST_SHUTDOWN_TIMER_SEC));
        let mut unlocked_socket = s4.lock().unwrap();
        tracing::debug!("[client] closing client websocket");
        unlocked_socket.close(Some(CloseFrame{ code: CloseCode::Normal, reason: Default::default() })).unwrap();

    }

    /// send a series of PINGs to the server; run on a separate thread to prove
    /// we can alternately read and write and that read doesn't block when there's nothing
    /// coming from the server.
    fn ping(ws_arc: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) {
        let max = 100;

        tracing::debug!("[client] spawned counter thread to {max}");
        for i in 0..max {
            tracing::debug!("[client] sending ping: {i}");
            // lock websocket
            {
                let mut unlocked_socket = ws_arc.lock().expect("[client] ping loop couldn't unlock");
                // tracing::debug!("[client] ws locked, sending count");

                match unlocked_socket.send(Message::Ping(vec![])){
                    // match unlocked_socket.send(Message::Text(format!("client count: {}", i))) {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("[client] send error, exiting ping thread: {:?}", e);
                        break;
                    }
                }
            }
            // websocket unlocked
            std::thread::sleep(Duration::from_secs(1));
        }
    }

}


