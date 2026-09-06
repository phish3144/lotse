import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Läuft als eigenständige Web-App (Cloudflare Pages) und eingebettet im
// Tauri-2-Webview (Desktop). Relative Basis, damit beide Auslieferungswege
// funktionieren, ohne die Konfiguration zu verzweigen.
export default defineConfig({
  base: './',
  plugins: [svelte()],
  test: {
    environment: 'happy-dom',
    include: ['src/**/*.{test,spec}.ts'],
  },
});
