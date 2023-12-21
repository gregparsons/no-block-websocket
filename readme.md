## Non-blocking Tungstenite websockets without Tokio

Usage:
```cargo test```

The test function demonstrates starting a server and client. Then it 
tells the client, via crossbeam channel, to start sending PINGs (simulating
sending a write command and demonstrating that the client's websocket read()
function does not block when there's nothing on the line for it to read). 
Both server or client can be told to shutdown from outside the websocket 
thread.

References:
- https://github.com/snapview/tungstenite-rs/issues/11
- https://www.reddit.com/r/rust/comments/dktiwf/reading_from_a_tcpstream_without_blocking/
