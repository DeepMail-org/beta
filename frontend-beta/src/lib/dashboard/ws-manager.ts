import { getStoredToken } from "@/lib/api";

type MessageCallback = (data: unknown) => void;

interface ChannelSubscription {
  callbacks: Set<MessageCallback>;
}

const MAX_RECONNECT_DELAY = 30_000;
const INITIAL_RECONNECT_DELAY = 1_000;

class WebSocketManager {
  private ws: WebSocket | null = null;
  private channels = new Map<string, ChannelSubscription>();
  private reconnectDelay = INITIAL_RECONNECT_DELAY;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private isIntentionallyClosed = false;
  private url: string | null = null;

  connect(baseUrl: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    this.isIntentionallyClosed = false;
    const token = getStoredToken();
    this.url = `${baseUrl}/ws/dashboard${token ? `?token=${encodeURIComponent(token)}` : ""}`;

    try {
      this.ws = new WebSocket(this.url);
    } catch {
      this.scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      this.reconnectDelay = INITIAL_RECONNECT_DELAY;
      for (const channel of this.channels.keys()) {
        this.sendSubscribe(channel);
      }
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data as string) as { channel?: string; data?: unknown };
        if (msg.channel && this.channels.has(msg.channel)) {
          const sub = this.channels.get(msg.channel)!;
          for (const cb of sub.callbacks) {
            cb(msg.data);
          }
        }
      } catch {
        // Malformed message — ignore
      }
    };

    this.ws.onclose = () => {
      this.ws = null;
      if (!this.isIntentionallyClosed) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = () => {
      this.ws?.close();
    };
  }

  subscribe(channel: string, callback: MessageCallback): () => void {
    if (!this.channels.has(channel)) {
      this.channels.set(channel, { callbacks: new Set() });
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.sendSubscribe(channel);
      }
    }

    const sub = this.channels.get(channel)!;
    sub.callbacks.add(callback);

    return () => {
      sub.callbacks.delete(callback);
      if (sub.callbacks.size === 0) {
        this.channels.delete(channel);
        if (this.ws?.readyState === WebSocket.OPEN) {
          this.sendUnsubscribe(channel);
        }
      }
    };
  }

  disconnect(): void {
    this.isIntentionallyClosed = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private sendSubscribe(channel: string): void {
    this.ws?.send(JSON.stringify({ action: "subscribe", channel }));
  }

  private sendUnsubscribe(channel: string): void {
    this.ws?.send(JSON.stringify({ action: "unsubscribe", channel }));
  }

  private scheduleReconnect(): void {
    if (this.isIntentionallyClosed) return;
    if (this.reconnectTimer) return;

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (this.url) {
        const baseUrl = this.url.split("/ws/dashboard")[0];
        this.connect(baseUrl);
      }
    }, this.reconnectDelay);

    this.reconnectDelay = Math.min(this.reconnectDelay * 2, MAX_RECONNECT_DELAY);
  }
}

export const wsManager = new WebSocketManager();
