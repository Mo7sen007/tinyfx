import type { MessageDto } from "@dto/message.dto";

export class Message {

  constructor(
    public username: string,
    public content: string,
  ) {}

  static from(dto: MessageDto): Message {
    return new Message(dto.username, dto.content);
  }
}
