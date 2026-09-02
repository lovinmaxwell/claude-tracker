<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let message = $state("…");

  async function load() {
    message = await invoke<string>("greet", { name: "Quota Tray" });
  }

  $effect(() => {
    void load();
  });
</script>

<main>
  <h1>Quota Tray</h1>
  <p>{message}</p>
</main>
