import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    // Vite's default browser baseline (Chrome 111 / Safari 16.4) fits every
    // webview Tauri 2 runs on; an older target only adds lab() fallbacks for
    // every oklch() theme color (+2 kB CSS).
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rolldownOptions: {
      // Pages other than the dashboard are loaded on their first visit; the
      // libraries every page needs get a chunk with a clear name.
      // Groups take their dependencies along: vendor must win over icons (Vue's h()).
      output: {
        codeSplitting: {
          groups: [
            { name: "vendor", test: /node_modules/, priority: 2 },
            { name: "icons", test: /src[\\/]icons/, priority: 1 },
          ],
        },
      },
    },
  },
});
