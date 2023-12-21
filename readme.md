## Non-blocking tungstenite websockets without Tokio

Usage:
```cargo test```

The test function demonstrates starting a server and client. Then it 
tells the client, via crossbeam channel, to start sending PINGs (simulating
sending a write command and demonstrating that the client's websocket read()
function does not block when there's nothing on the line for it to read). 
Both server or client can be told to shutdown from outside the websocket 
thread.