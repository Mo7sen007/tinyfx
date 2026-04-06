// Message service

import { createHttp } from "@tinyfx/runtime";
import { MessageDto } from "@dto/message.dto";
import { createWebsocketService } from "./websocket.service";

const http = createHttp({ baseUrl: "http://localhost:8080/api/" });

export function createMessageService() {
  const socket = createWebsocketService();
  return {
    getHealth: () => http.get("health"),
    getMessages: () => http.get<MessageDto[]>("messages/"),
    sendMessage: (message: MessageDto) => http.post<MessageDto>("messages/",  message ),
    onMessage: (cb: (msg: MessageDto) => void) => {
      socket.onMessage(cb);
    },
  };
}

export type MessageService = ReturnType<typeof createMessageService>;
