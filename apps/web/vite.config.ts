import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Läuft als eigenständige Web-App (Cloudflare Pages) und eingebettet im
// Tauri-2-Webview (Desktop). Relative Basis, damit beide Auslieferungswege
// funktionieren, ohne die Konfiguration zu verzweigen.
export default defineConfig({
  base: './',
  plugins: [svelte()],
  // Für Tests die Browser-Variante von Svelte auflösen. Ohne das landet `mount()` im
  // Server-Build und wirft »not available on the server« – Komponenten liessen sich dann
  // gar nicht prüfen, und genau das hat in 0.8.0 einen kaputten Dialog durchgelassen.
  resolve: process.env.VITEST ? { conditions: ['browser'] } : undefined,
  test: {
    environment: 'happy-dom',
    include: ['src/**/*.{test,spec}.ts'],
  },
});
