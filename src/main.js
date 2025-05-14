const { invoke } = window.__TAURI__.core;

let chatInputEl;
let chatMsgEl;

async function submit() {
  try {
    const url = new URL(chatInputEl.value);
    let streamId = null;

    if (url.pathname === "/watch") {
      streamId = url.searchParams.get("v");
    }

    if (url.pathname.startsWith("/live/")) {
      streamId = url.pathname.replace("/live/", "")
    }

    if (!streamId) {
      throw new Error("Wrong path")
    }

    console.log('invoking...')
    await invoke("open_chat", { streamId });
  } catch(e) {
    alert('Invalid url!')
  }
}

window.addEventListener("DOMContentLoaded", () => {
  chatInputEl = document.querySelector("#chat-input");
  chatMsgEl = document.querySelector("#chat-msg");
  document.querySelector("#chat-form").addEventListener("submit", submit);
});
