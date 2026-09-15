import { mount } from 'svelte';
import './app.css';
import App from './App.svelte';
import { anwenden, thema } from './lib/thema.svelte';

// Vor dem Aufbau, damit niemand kurz das falsche Thema sieht.
anwenden(thema.wert);

const target = document.getElementById('app');
if (!target) {
  throw new Error('Root-Element #app nicht gefunden.');
}

export default mount(App, { target });
