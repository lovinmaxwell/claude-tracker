import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  base: "./",
  define: {
    "import.meta.env.QUOTA_TRAY_PLATFORM": JSON.stringify("chrome"),
  },
  build: {
    outDir: "dist-extension",
    emptyOutDir: true,
    target: "chrome109",
    minify: "esbuild",
    sourcemap: false,
    modulePreload: { polyfill: false },
    rollupOptions: {
      input: {
        popup: resolve(__dirname, "index.html"),
        settings: resolve(__dirname, "settings.html"),
        background: resolve(__dirname, "src/extension/background.ts"),
      },
      output: {
        entryFileNames: (chunk) =>
          chunk.name === "background" ? "background.js" : "assets/[name]-[hash].js",
        chunkFileNames: "assets/[name]-[hash].js",
        assetFileNames: "assets/[name]-[hash][extname]",
      },
    },
  },
});
