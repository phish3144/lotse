<script lang="ts">
  // Globale Schnellerfassung: ein einzelnes Eingabefeld, Enter speichert sofort.
  // "@projektname" ordnet einem Projekt zu, sonst landet der Gedanke im Postkorb –
  // nie in einem Dialog (CONCEPT.md Abschnitt 4). Strg/Cmd+K öffnet und schließt.
  import { provider } from '../data/store';
  import { datenVersion } from '../data/version.svelte';
  import { erfassung } from '../erfassung.svelte';
  import { ART_LABEL } from '../format';
  import type { NotizArt, Projekt } from '../data/types';

  const ARTEN: NotizArt[] = ['log', 'offen', 'entscheidung'];

  let eingabe = $state('');
  let wirdGespeichert = $state(false);
  let fehler: string | null = $state(null);
  let projekte: Projekt[] = $state([]);
  let inputEl: HTMLInputElement | undefined = $state();

  // Beim Öffnen: Vorbelegung übernehmen, Projektliste für die @-Auflösung holen.
  $effect(() => {
    if (!erfassung.offen) return;
    eingabe = erfassung.vorbelegung;
    fehler = null;
    void provider.listProjects().then((liste) => (projekte = liste));
  });

  $effect(() => {
    if (erfassung.offen && inputEl) {
      inputEl.focus();
      // Cursor ans Ende, damit man hinter "@Projekt " einfach weitertippt.
      const laenge = inputEl.value.length;
      inputEl.setSelectionRange(laenge, laenge);
    }
  });

  function aufTastenkuerzel(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      erfassung.umschalten();
    } else if (e.key === 'Escape' && erfassung.offen) {
      erfassung.schliessen();
    }
  }

  /** Löst "@name rest" gegen die Projektliste auf. Ohne Treffer bleibt der Text ganz. */
  function ziel(text: string): { projekt?: Projekt; text: string } {
    const treffer = text.match(/^@(\S+)\s*([\s\S]*)$/);
    if (!treffer) return { text };
    const name = treffer[1].toLowerCase();
    const rest = treffer[2].trim();
    const passt =
      projekte.find((p) => p.titel.toLowerCase() === name) ??
      projekte.find((p) => p.titel.toLowerCase().startsWith(name)) ??
      projekte.find((p) => p.titel.toLowerCase().includes(name));
    if (!passt) return { text };
    return { projekt: passt, text: rest || text };
  }

  const aufloesung = $derived(ziel(eingabe.trim()));
  const zielTitel = $derived(eingabe.trim() ? (aufloesung.projekt?.titel ?? 'Postkorb') : '');

  async function speichern() {
    const roh = eingabe.trim();
    if (!roh || wirdGespeichert) return;
    wirdGespeichert = true;
    fehler = null;
    try {
      const { projekt, text } = ziel(roh);
      // Ohne Zuordnung: Auffangprojekt, das der Kern bei Bedarf selbst anlegt.
      const zielProjekt = projekt ?? (await provider.postkorb());
      await provider.addNote(zielProjekt.id, { quelle: 'mensch', art: erfassung.art, text });
      datenVersion.bump();
      eingabe = '';
      erfassung.schliessen();
    } catch (e) {
      fehler = e instanceof Error ? e.message : String(e);
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

{#if erfassung.offen}
  <div class="ueberlagerung">
    <button type="button" class="rueckwand" aria-label="Schnellerfassung schließen" onclick={() => erfassung.schliessen()}
    ></button>
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Schnellerfassung">
      <input
        bind:this={inputEl}
        bind:value={eingabe}
        onkeydown={aufEingabeTaste}
        type="text"
        placeholder="Was gibt's? @projekt für Zuordnung, sonst Postkorb …"
        aria-label="Schnellerfassung"
      />
      <div class="arten" role="group" aria-label="Art des Eintrags">
        {#each ARTEN as art (art)}
          <button type="button" class:gewaehlt={erfassung.art === art} onclick={() => (erfassung.art = art)}>
            {ART_LABEL[art]}
          </button>
        {/each}
      </div>
      {#if fehler}
        <p class="fehler">{fehler}</p>
      {/if}
      <div class="fusszeile">
        <span class="ziel-hinweis">{zielTitel ? `→ ${zielTitel}` : ''}</span>
        <span class="hinweis">{wirdGespeichert ? 'Speichere …' : 'Enter zum Speichern · Esc zum Schließen'}</span>
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
  .arten {
    display: flex;
    gap: 0.35rem;
    margin-top: 0.6rem;
  }
  .arten button {
    border: 1px solid var(--rahmen);
    background: transparent;
    color: var(--text-gedaempft);
    border-radius: 999px;
    padding: 0.15rem 0.7rem;
    font-size: 0.8rem;
  }
  .arten button.gewaehlt {
    border-color: var(--akzent);
    color: var(--akzent);
    font-weight: 600;
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
  .fehler {
    color: var(--farbe-ueberfaellig);
    font-size: 0.85rem;
    margin: 0.5rem 0 0;
  }
</style>
