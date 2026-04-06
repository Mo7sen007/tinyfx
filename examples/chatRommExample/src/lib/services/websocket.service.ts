// Websocket service


export function createWebsocketService() {
  const socket = new WebSocket("ws://localhost:8080/api/messages/ws");

  const listeners: ((msg: any) => void)[] = [];

  socket.onopen = () => {
    console.log("Connected");
  };

  socket.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    console.log("Received:", msg);

    // notify all subscribers
    listeners.forEach((cb) => cb(msg));
  };

  socket.onclose = () => {
    console.log("Disconnected");
  };

  socket.onerror = (err) => {
    console.error("Socket error:", err);
  };

  return {
    send(data: any) {
      socket.send(JSON.stringify(data));
    },

    onMessage(cb: (msg: any) => void) {
      listeners.push(cb);
    },

    close() {
      socket.close();
    },
  };
}