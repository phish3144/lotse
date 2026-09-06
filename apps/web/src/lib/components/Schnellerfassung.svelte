<script lang="ts">
  // Globale Schnellerfassung: Strg/Cmd+K öffnet ein einzelnes Eingabefeld.
  // "@projektname" ordnet einem Projekt zu, sonst landet der Gedanke im Postkorb.
  // Enter speichert sofort über den Provider, kein Dialog davor (CONCEPT.md Abschnitt 4).
  import { provider, POSTKORB_PROJEKT_ID } from '../data/store';
  import { datenVersion } from '../data/version.svelte';
  import type { Projekt } from '../data/types';

  let offen = $state(false);
  let eingabe = $state('');
  let wirdGespeichert = $state(false);
  let projekte: Projekt[] = $state([]);
  let zielTitel = $state('');
  let inputEl: HTMLInputElement | undefined = $state();

  async function oeffnen() {
    offen = true;
    eingabe = '';
    zielTitel = '';
    projekte = await provider.listProjects();
  }

  function schliessen() {
    offen = false;
  }

  function aufTastenkuerzel(e: KeyboardEvent) {
    const istModifier = e.metaKey || e.ctrlKey;
    if (istModifier && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      if (offen) {
        schliessen();
      } else {
        void oeffnen();
      }
    } else if (e.key === 'Escape' && offen) {
      schliessen();
    }
  }

  $effect(() => {
    if (offen) {
      inputEl?.focus();
    }
  });

  function ziel(text: string): { projektId: string; text: string; projektTitel: string } {
    if (text.startsWith('@')) {
      const [, roheProjektangabe, ...rest] = text.match(/^@(\S+)\s*(.*)$/s) ?? [];
      const name = (roheProjektangabe ?? '').toLowerCase();
      const gefunden = projekte.find((p) => p.titel.toLowerCase().includes(name));
      if (gefunden) {
        return { projektId: gefunden.id, text: rest.join(' ').trim() || text, projektTitel: gefunden.titel };
      }
    }
    return { projektId: POSTKORB_PROJEKT_ID, text, projektTitel: 'Postkorb' };
  }

  $effect(() => {
    zielTitel = eingabe.trim() ? ziel(eingabe.trim()).projektTitel : '';
  });

  async function speichern() {
    const text = eingabe.trim();
    if (!text || wirdGespeichert) return;
    const { projektId, text: notizText } = ziel(text);
    wirdGespeichert = true;
    try {
      await provider.addNote(projektId, { quelle: 'mensch', art: 'log', text: notizText });
      datenVersion.bump();
      schliessen();
    } finally {
      wirdGespeichert = false;
    }
  }

  function aufEingabeTaste(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void speichern();
    }
  }
</script>

<svelte:window onkeydown={aufTastenkuerzel} />

{#if offen}
  <div class="ueberlagerung">
    <button type="button" class="rueckwand" aria-label="Schnellerfassung schließen" onclick={schliessen}></button>
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Schnellerfassung">
      <input
        bind:this={inputEl}
        bind:value={eingabe}
        onkeydown={aufEingabeTaste}
        type="text"
        placeholder="Was gibt's? @projekt für Zuordnung, sonst Postkorb …"
        aria-label="Schnellerfassung"
      />
      <div class="fusszeile">
        <span class="ziel-hinweis">{zielTitel ? `→ ${zielTitel}` : ''}</span>
        <span class="hinweis">Enter zum Speichern · Esc zum Schließen</span>
      </div>
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
    padding: 0.9rem 1rem;
    width: min(34rem, 92vw);
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
  .fusszeile {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-top: 0.5rem;
    font-size: 0.8rem;
  }
  .ziel-hinweis {
    color: var(--akzent);
    font-weight: 600;
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
</style>
