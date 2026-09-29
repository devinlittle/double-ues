const API_URL = "http://10.0.56.41:5252";

import type { components } from "./types/chat.api";
type ChatMessage = components["schemas"]["ChatMessage"];
type ClientType = components["schemas"]["ClientType"];

//! STATE BEGIN
export let chat_messages = $state<ChatMessage[]>([]);
//! STATE END

type SocketState = "connected" | "connecting" | "disconnected" | "initial_page";
type SocketRole = "sender" | "receiver";

function updateSocketState(
  newStatus: "connected" | "connecting" | "disconnected",
  socketState: SocketState,
  role: SocketRole
) {
  socketState = newStatus;

  const label = role === "sender" ? "sender chat" : "receiver chat";
  switch (newStatus) {
    case "disconnected":
      console.error(`[double_ues]: disconnected from ${label}`);
      break;
    case "connecting":
      console.log(`[double_ues]: connecting to the ${label}`);
      break;
    case "connected":
      console.log(`[double_ues]: connected to the ${label}`);
      break;
  }

}


//! CHAT RECEIVER STUFF BEGIN
let receiverSocket: WebSocket | null = null;
let _receiverState = $state<SocketState>("initial_page");

export const receiverState = {
  get value() {
    return _receiverState;
  },
  set value(v) {
    _receiverState = v;
  },
};

export async function receive_from_chat() {
  if (receiverSocket) {
    receiverSocket.onopen = null;
    receiverSocket.onmessage = null;
    receiverSocket.onerror = null;
    receiverSocket.onclose = null;

    if (
      receiverSocket.readyState === WebSocket.OPEN ||
      receiverSocket.readyState === WebSocket.CONNECTING
    ) {
      receiverSocket.close();
    }
    receiverSocket = null;
  }

  let client_type: ClientType = "ChatFrontend";
  const url = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/ws/${client_type}`;

  const ws = new WebSocket(url);
  receiverSocket = ws;

  updateSocketState("connecting", _receiverState, "receiver");

  ws.onopen = () => {
    if (ws !== receiverSocket) return;

    updateSocketState("connected", _receiverState, "receiver");
  };

  ws.onmessage = (e: MessageEvent) => {
    // console.log(`CHAT MESSAGE: ${e.data}`);

    let ws_message: ChatMessage = JSON.parse(e.data);
    chat_messages.push(ws_message);
  };

  ws.onclose = () => {
    if (ws !== receiverSocket) return;
    receiverSocket = null;

    updateSocketState("disconnected", _receiverState, "receiver");
  };

  ws.onerror = () => {
    receive_from_chat();
  };
}
//! CHAT RECEIVER STUFF END

//! you send? WS BEGIN
type OnboardinInfo = components["schemas"]["UserFields"];
type OkayMessage = components["schemas"]["OkayMessage"];
type ThisIsOkay = components["schemas"]["ThisIsOk"];
const OK: ThisIsOkay = "OK";

let okayMessage: OkayMessage = {
  ok: OK
}

let senderSocket: WebSocket | null = null;
let _senderState = $state<SocketState>("initial_page");

export const senderState = {
  get value() {
    return _senderState;
  },
  set value(v) {
    _senderState = v;
  },
};

export async function connect_to_quick_add() {
  if (senderSocket) {
    senderSocket.onopen = null;
    senderSocket.onmessage = null;
    senderSocket.onerror = null;
    senderSocket.onclose = null;

    if (
      senderSocket.readyState === WebSocket.OPEN ||
      senderSocket.readyState === WebSocket.CONNECTING
    ) {
      senderSocket.close();
    }
    senderSocket = null;
  }

  let client_type: ClientType = "User";
  const url = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/ws/${client_type}`;

  const ws = new WebSocket(url);
  senderSocket = ws;

  updateSocketState("connecting", _senderState, "sender");

  ws.onopen = () => {
    if (ws !== senderSocket) return;

    updateSocketState("connected", _senderState, "sender");
  };

  ws.onmessage = (e: MessageEvent) => {
    if (e.data === JSON.stringify(okayMessage)) {
      console.log("ALRIGHT TO CONTINUE")
    }
  };

  ws.onclose = () => {
    if (ws !== senderSocket) return;
    senderSocket = null;

    updateSocketState("disconnected", _senderState, "sender");
  };

  ws.onerror = () => {
    connect_to_quick_add();
  };
}

export function getSenderSocket(): WebSocket | null {
  return senderSocket;
}


export function socketSend(msg: string) {
  const socket = getSenderSocket();

  if (socket === null) return

  socket?.send(`${msg}`);
}

export function sendOnboardingInfo(info: OnboardinInfo): boolean {
  const socket = getSenderSocket();

  if (socket === null) return false;

  const json_to_send = JSON.stringify(info);

  socket?.send(`im: ${json_to_send}`);
  return true;
}

//! you send? WS END
