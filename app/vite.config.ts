// `vitest/config` re-exports Vite's own `defineConfig`, widened to accept the
// `test` block below. One config file, so the tests and the app resolve
// imports identically.
import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri serves the UI from a fixed port in development and from `dist/` in a
// bundle. The port is pinned because `tauri.conf.json` points the window at
// it; `strictPort` makes a clash an error rather than a silently different
// URL that the window then fails to load.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // The Rust side has its own rebuild loop; watching it here only causes
      // the page to reload while the backend is mid-compile.
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // Matches the oldest webview Tauri 2 supports on each platform.
    target: ["es2021", "chrome100", "safari15"],
    sourcemap: true,
    // The bundle is read from the application's own files, never over a
    // network, so Vite's 500 kB warning (written for web pages) is noise;
    // CodeMirror alone is most of a megabyte.
    chunkSizeWarningLimit: 2000,
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
