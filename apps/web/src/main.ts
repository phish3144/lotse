import { mount } from 'svelte';
import './app.css';
import App from './App.svelte';

const target = document.getElementById('app');
if (!target) {
  throw new Error('Root-Element #app nicht gefunden.');
}

export default mount(App, { target });
