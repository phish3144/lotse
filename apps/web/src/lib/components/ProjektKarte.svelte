<script lang="ts">
  import type { Auffaelligkeit } from '../brief';
  import type { Notiz, Projekt } from '../data/types';
  import { alterInTagenText } from '../format';
  import AuffaelligkeitPunkt from './AuffaelligkeitPunkt.svelte';

  let {
    projekt,
    letzteNotiz,
    auffaelligkeit,
  }: { projekt: Projekt; letzteNotiz?: Notiz; auffaelligkeit: Auffaelligkeit } = $props();
</script>

<a class="karte" href={`#/projekt/${projekt.id}`}>
  <div class="kopf">
    <AuffaelligkeitPunkt wert={auffaelligkeit} />
    <h3>{projekt.titel}</h3>
  </div>
  <p class="kurs">{projekt.kurs}</p>
  {#if letzteNotiz}
    <p class="letzte-notiz">
      <span class="notiz-text">{letzteNotiz.text.split('\n')[0]}</span>
      <span class="alter">· {alterInTagenText(letzteNotiz.ts)}</span>
    </p>
  {:else}
    <p class="letzte-notiz alter">Noch keine Logbuch-Einträge.</p>
  {/if}
</a>

<style>
  .karte {
    display: block;
    padding: 0.9rem 1rem;
    border: 1px solid var(--rahmen);
    border-radius: 0.6rem;
    background: var(--karten-hintergrund);
    color: inherit;
    text-decoration: none;
    transition: border-color 0.15s ease;
  }
  .karte:hover,
  .karte:focus-visible {
    border-color: var(--akzent);
  }
  .kopf {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
  .kurs {
    margin: 0.35rem 0 0.5rem;
    color: var(--text-gedaempft);
    font-size: 0.9rem;
  }
  .letzte-notiz {
    margin: 0;
    font-size: 0.85rem;
    display: flex;
    gap: 0.35em;
    flex-wrap: wrap;
  }
  .notiz-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
  .alter {
    color: var(--text-gedaempft);
    white-space: nowrap;
  }
</style>
