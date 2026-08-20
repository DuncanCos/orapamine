// Client WebSocket : connexion, envoi typé, et reconnexion automatique
// avec backoff si la connexion tombe après qu'on ait un `token` de joueur
// (rechargement de page, coupure réseau — voir plan §5).

import type { ClientMsg, ServerMsg } from "../types/protocol";

type Listener = (msg: ServerMsg) => void;
type ConnListener = (connected: boolean) => void;

const RECONNECT_DELAYS_MS = [500, 1000, 2000, 4000, 8000];

export class GameSocket {
  private ws: WebSocket | null = null;
  private listeners = new Set<Listener>();
  private connListeners = new Set<ConnListener>();
  private token: string | null = null;
  private reconnectAttempt = 0;
  private closedByUser = false;
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  connect(): void {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }
    this.closedByUser = false;
    this.ws = new WebSocket(this.url);
    this.ws.onopen = () => {
      this.reconnectAttempt = 0;
      this.connListeners.forEach((l) => l(true));
      if (this.token) {
        this.send({ type: "Reconnect", token: this.token });
      }
    };
    this.ws.onmessage = (ev) => {
      const msg: ServerMsg = JSON.parse(ev.data);
      if (msg.type === "GameCreated" || msg.type === "Joined") {
        this.token = msg.token;
        localStorage.setItem("orapamine_token", msg.token);
      }
      this.listeners.forEach((l) => l(msg));
    };
    this.ws.onclose = () => {
      this.connListeners.forEach((l) => l(false));
      if (!this.closedByUser && this.token) {
        this.scheduleReconnect();
      }
    };
    this.ws.onerror = () => {
      this.ws?.close();
    };
  }

  private scheduleReconnect(): void {
    const delay = RECONNECT_DELAYS_MS[Math.min(this.reconnectAttempt, RECONNECT_DELAYS_MS.length - 1)];
    this.reconnectAttempt += 1;
    setTimeout(() => {
      if (!this.closedByUser) this.connect();
    }, delay);
  }

  restoreToken(): string | null {
    this.token = localStorage.getItem("orapamine_token");
    return this.token;
  }

  send(msg: ClientMsg): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }

  onMessage(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  onConnectionChange(listener: ConnListener): () => void {
    this.connListeners.add(listener);
    return () => this.connListeners.delete(listener);
  }

  close(): void {
    this.closedByUser = true;
    this.ws?.close();
  }

  /** Quitte définitivement la partie en cours (retour au menu) : oublie le
   * jeton (plus de reconnexion possible à cette partie), ferme la
   * connexion, puis en rouvre aussitôt une neuve et vierge pour permettre
   * de créer/rejoindre une autre partie. */
  leaveGame(): void {
    this.token = null;
    localStorage.removeItem("orapamine_token");
    this.close();
    this.connect();
  }
}

export function defaultWsUrl(): string {
  const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${proto}//${window.location.host}/ws`;
}
