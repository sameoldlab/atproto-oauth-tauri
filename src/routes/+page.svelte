<script lang="ts">
  import {
    cancel,
    onInvalidUrl,
    onUrl,
    start,
  } from "@fabianlars/tauri-plugin-oauth";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  let name = $state("");
  let greetMsg = $state("");
  let message = $state("");

  let currentPort: number | null = $state(null);
  let isRustServer = false;

  const webview = getCurrentWebviewWindow();
  webview.listen("redirect_uri", (e) =>
    console.log(`received redirect event`, e),
  );
  async function stopAuthServer() {
    if (currentPort !== null) {
      try {
        if (isRustServer) await invoke("stop_server", { port: currentPort });
        else await cancel(currentPort);
        console.log(`Stopped server on port ${currentPort}`);
      } catch (error) {
        console.error(`Error stopping server: ${error}`);
      }
      currentPort = null;
    }
  }

  async function startAuthServer() {
    await stopAuthServer();
    try {
      const port = await invoke<number>("start_server");
      currentPort = port;
      message = `OAuth server started on port ${port} (Rust)`;
    } catch (error) {
      message = `Error starting OAuth server (Rust): ${error}`;
    }
  }

  async function startServerTS() {
    await stopAuthServer();
    try {
      const port = await start();
      currentPort = port;
      isRustServer = false;
      message = `OAuth server started on port ${port} (TypeScript)`;

      const unlistenUrl = await onUrl((url) => {
        console.log("Received OAuth URL:", url);
        message += `\nReceived OAuth URL: ${url}`;
      });

      const unlistenInvalidUrl = await onInvalidUrl((error) => {
        console.error("Received invalid OAuth URL:", error);
        message += `\nReceived invalid OAuth URL: ${error}`;
      });

      // Store unlisten functions to call them when stopping the server
      (window as any).unlistenFunctions = [unlistenUrl, unlistenInvalidUrl];
    } catch (error) {
      message = `Error starting OAuth server (TypeScript): ${error}`;
    }
  }
  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="container">
  <h1>Welcome to Tauri + Svelte</h1>

  <div class="row">
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://kit.svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte-kit" alt="SvelteKit Logo" />
    </a>
  </div>
  <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <form class="row" onsubmit={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>
  <p><strong>{message}</strong></p>
  <button onclick={startAuthServer}>Start Flow</button>
  <button onclick={startServerTS}>Start Flow TS</button>
  <br />
  {#if currentPort}
    <button onclick={() => stopAuthServer()}>Stop Server</button>
  {/if}
</main>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte-kit:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }

  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: 0.75s;
  }

  .logo.tauri:hover {
    filter: drop-shadow(0 0 2em #24c8db);
  }

  .row {
    display: flex;
    justify-content: center;
  }

  a {
    font-weight: 500;
    color: #646cff;
    text-decoration: inherit;
  }

  a:hover {
    color: #535bf2;
  }

  h1 {
    text-align: center;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  input,
  button {
    outline: none;
  }

  #greet-input {
    margin-right: 5px;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    a:hover {
      color: #24c8db;
    }

    input,
    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
