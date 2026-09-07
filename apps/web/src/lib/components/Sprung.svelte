<script lang="ts">
  // Sprung zu einem Projekt, Strg/Cmd+P. Tippen filtert, Pfeiltasten wählen,
  // Enter springt. Bei zwölf Projekten ist das der schnellste Weg; über die
  // Hafenkarten zu klicken ist es nicht.
  import { provider } from '../data/store';
  import { STATUS_LABEL } from '../format';
  import { navigiereZu } from '../router.svelte';
  import { sprung } from '../sprung.svelte';
  import type { Projekt } from '../data/types';

  const MAX = 8;

  let projekte: Projekt[] = $state([]);
  let eingabe = $state('');
  let markiert = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (!sprung.offen) return;
    eingabe = '';
    markiert = 0;
    void provider.listProjects().then((liste) => (projekte = liste));
  });

  $effect(() => {
    if (sprung.offen) inputEl?.focus();
  });

  /** Buchstaben in Reihenfolge, nicht zwingend zusammenhängend – „gtk“ findet „Getränkekasse“. */
  function passt(titel: string, suche: string): boolean {
    if (!suche) return true;
    const t = titel.toLowerCase();
    let i = 0;
    for (const zeichen of suche.toLowerCase()) {
      i = t.indexOf(zeichen, i);
      if (i < 0) return false;
      i += 1;
    }
    return true;
  }

  const treffer = $derived(
    projekte
      .filter((p) => passt(p.titel, eingabe.trim()))
      .sort((a, b) => {
        const s = eingabe.trim().toLowerCase();
        // Wer vorne passt, steht vorne; sonst das zuletzt Berührte.
        const av = s && a.titel.toLowerCase().startsWith(s) ? 0 : 1;
        const bv = s && b.titel.toLowerCase().startsWith(s) ? 0 : 1;
        return av - bv || b.zuletzt_beruehrt.localeCompare(a.zuletzt_beruehrt);
      })
      .slice(0, MAX),
  );

  $effect(() => {
    if (markiert >= treffer.length) markiert = Math.max(0, treffer.length - 1);
  });

  function springe(p: Projekt | undefined) {
    if (!p) return;
    sprung.schliessen();
    navigiereZu(`#/projekt/${p.id}`);
  }

  function aufTaste(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      markiert = (markiert + 1) % Math.max(1, treffer.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      markiert = (markiert - 1 + treffer.length) % Math.max(1, treffer.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      springe(treffer[markiert]);
    }
  }
</script>

{#if sprung.offen}
  <div class="ueberlagerung">
    <button type="button" class="rueckwand" aria-label="Schließen" onclick={() => sprung.schliessen()}></button>
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Zu Projekt springen">
      <input
        bind:this={inputEl}
        bind:value={eingabe}
        onkeydown={aufTaste}
        type="text"
        placeholder="Zu Projekt springen …"
        aria-label="Projekt suchen"
      />
      {#if treffer.length === 0}
        <p class="leer">Kein Projekt passt.</p>
      {:else}
        <ul>
          {#each treffer as p, i (p.id)}
            <li>
              <button type="button" class:markiert={i === markiert} onmouseenter={() => (markiert = i)} onclick={() => springe(p)}>
                <span class="titel">{p.titel}</span>
                <span class="status">{STATUS_LABEL[p.status]}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      <p class="fuss">↑↓ wählen · Enter springt · Esc schließt</p>
    </div>
  </div>
{/if}

<style>
  .ueberlagerung {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 12vh;
    z-index: 100;
  }
  .rueckwand {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border: none;
    padding: 0;
    background: rgba(0, 0, 0, 0.35);
    cursor: default;
  }
  .dialog {
    position: relative;
    z-index: 1;
    background: var(--karten-hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.7rem;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3);
    padding: 0.9rem 1rem 0.7rem;
    width: min(30rem, 92vw);
  }
  .dialog input {
    width: 100%;
    box-sizing: border-box;
    font-size: 1.05rem;
    padding: 0.5rem 0.2rem;
    border: none;
    border-bottom: 1px solid var(--rahmen);
    background: transparent;
    color: inherit;
  }
  .dialog input:focus {
    outline: none;
    border-bottom-color: var(--akzent);
  }
  ul {
    list-style: none;
    margin: 0.5rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  ul button {
    display: flex;
    width: 100%;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    text-align: left;
    font: inherit;
    border: none;
    background: transparent;
    color: inherit;
    border-radius: 0.35rem;
    padding: 0.4rem 0.5rem;
  }
  ul button.markiert {
    background: var(--hintergrund);
  }
  .titel {
    font-weight: 600;
  }
  .status {
    font-size: 0.8rem;
    color: var(--text-gedaempft);
    white-space: nowrap;
  }
  .leer,
  .fuss {
    color: var(--text-gedaempft);
    font-size: 0.8rem;
    margin: 0.6rem 0 0;
  }
  .fuss {
    text-align: right;
  }
</style>
