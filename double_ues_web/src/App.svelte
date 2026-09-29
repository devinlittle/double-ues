<script lang="ts">
  // RECEIVER STUFF BEGIN
  import { onMount } from "svelte";
  import {
    receive_from_chat,
    chat_messages,
    connect_to_quick_add,
    sendOnboardingInfo,
    socketSend,
  } from "./lib/chat_handler.svelte";

  import type { components } from "./lib/types/chat.api";
  type RgbColor = components["schemas"]["RgbColor"];

  //  $inspect(chat_messages);

  onMount(async () => {
    await receive_from_chat();
  });
  // RECEIVER STUFF END

  // you send? BEGIN
  onMount(async () => {
    await connect_to_quick_add();
  });

  let onboarded = $state<boolean>(false);
  let username = $state<string>("");
  let username_color = $state("#ff0000");
  let message_font = $state<string>("");

  function fullHex(hex: string): RgbColor {
    let r = hex.slice(1, 2);
    let g = hex.slice(2, 3);
    let b = hex.slice(3, 4);

    return [parseInt(r + r, 16), parseInt(g + g, 16), parseInt(b + b, 16)];
  }

  function hex2rgb(hex: string): RgbColor {
    if (hex.length === 4) {
      return fullHex(hex);
    }

    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);

    // return {r, g, b}
    return [r, g, b];
  }

  const doTheSend = () => {
    if (
      sendOnboardingInfo({
        username: username,
        username_color: hex2rgb(username_color),
        message_font: message_font,
      })
    ) {
      onboarded = true;
    }
  };

  let messageToSend = $state<string>("");

  const sendMessage = () => {
    socketSend(messageToSend);
  };

  // you send? END
</script>

{#each chat_messages as chat}
  <div>
    <span style="color: rgb({chat.username_color.join(', ')});">
      {chat.username}:
    </span>
    <span style="font-family: {chat.message_font};">
      {chat.message}
    </span>
  </div>
{/each}

{#if !onboarded}
  <input type="color" bind:value={username_color} />
  <input type="text" bind:value={username} placeholder="username" />
  <input type="text" bind:value={message_font} placeholder="font" />
  <button onclick={doTheSend}>PLEASE CLICK</button>
{:else}
  <div class="textBox">
    <form
      onsubmit={(e) => {
        e.preventDefault();
      }}
    >
      <input type="text" bind:value={messageToSend} />
      <button onclick={sendMessage}>=></button>
    </form>
  </div>
{/if}
