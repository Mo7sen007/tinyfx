// Chat component
// Chat component

import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";
import type { MessageDto } from "@dto/message.dto";
import { createMessageService } from "@services/message.service";

export class Chat {
  public messages = signal<MessageDto[]>([]);

  private healthStatus = signal(false);
  private currentUsername = signal("currentUser");
  private messageService = createMessageService();

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  private addMessage(msg: MessageDto): void {
    const withOwn = {
      ...msg,
      own: msg.username === this.currentUsername() ? "sent" : "received",
    };

    // append instead of replacing
    this.messages.set([...this.messages(), withOwn]);
  }

  private async sendMessage(): Promise<void> {
    const input = this.el.querySelector<HTMLInputElement>("#content");
    if (!input) return;

    const messageText = input.value.trim();
    if (!messageText) return;

    const message: MessageDto = {
      username: this.currentUsername(),
      content: messageText,
    };

    try {
      // send via HTTP
      await this.messageService.sendMessage(message);

      input.value = "";
      // ❌ no fetch anymore
    } catch (err) {
      console.error("[Chat] Failed to send message:", err);
    }
  }
  public deleteMessage(id: number | string): void {
    this.messages.set(this.messages().filter((message) => message.id !== Number(id)));
  }

  private async checkHealth(): Promise<void> {
    const healthEl = this.el.querySelector<HTMLElement>("#health-status");
    if (!healthEl) return;

    try {
      await this.messageService.getHealth();
      this.healthStatus.set(true);
      healthEl.style.background = "#10b981";
    } catch {
      this.healthStatus.set(false);
      healthEl.style.background = "#ef4444";
    }
  }

  init(): void {
    console.log("Chat mounted", this.el);

    // username input
    this.el.querySelector<HTMLInputElement>("#username")
      ?.addEventListener("input", (e) => {
        this.currentUsername.set((e.target as HTMLInputElement).value);
      });

    // send button
    this.el.querySelector<HTMLButtonElement>("#send-btn")
      ?.addEventListener("click", () => void this.sendMessage());

    // subscribe to websocket messages 🔥
    this.messageService.onMessage((msg) => {
      this.addMessage(msg);
    });

    // initial health check
    void this.checkHealth();

    // OPTIONAL: initial load (so you don’t start empty)
    this.messageService.getMessages().then((msgs) => {
      const formatted = msgs.map(msg => ({
        ...msg,
        own: msg.username === this.currentUsername() ? "sent" : "received",
      }));
      this.messages.set(formatted);
    });
  }

  destroy(): void {
    // later: you can close websocket here if needed
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new Chat(el, ctx);
  inst.init();
  return inst;
}
