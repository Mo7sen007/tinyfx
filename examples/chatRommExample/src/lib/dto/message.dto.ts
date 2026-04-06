export interface MessageDto {
  id?: number;
  username: string;
  content: string;
  own?: string; // "sent" | "received" — computed on client
}