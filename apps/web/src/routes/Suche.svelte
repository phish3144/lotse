<script lang="ts">
  // Sucht beim Tippen. Die Adresse wird dabei nur ersetzt, nicht fortgeschrieben,
  // sonst führt jeder Tastendruck einen Schritt in den Zurück-Weg ein.
  import { provider } from '../lib/data/store';
  import type { Notiz, Projekt, Referenz, TresorEintrag } from '../lib/data/types';
  import { alterInTagenText, hervorheben, PRUEFSTATUS_LABEL, zeileMitTreffer } from '../lib/format';
  import { ersetze } from '../lib/router.svelte';

  type Art = 'alles' | 'projekt' | 'notiz' | 'referenz' | 'zugang';

  const ARTEN = [
    { id: 'alles', name: 'Alles' },
    { id: 'projekt', name: 'Projekte' },
    { id: 'notiz', name: 'Logbuch' },
    { id: 'referenz', name: 'Referenzen' },
    { id: 'zugang', name: 'Zugänge' },
  ] as const satisfies readonly { id: Art; name: string }[];

  const ART_NAME: Record<Exclude<Art, 'alles'>, string> = {
    projekt: 'Projekt',
    notiz: 'Logbuch',
    referenz: 'Referenz',
    zugang: 'Zugang',
  };

  let art: Art = $state('alles');

  interface Treffer {
    art: Exclude<Art, 'alles'>;
    id: string;
    href: string;
    titel?: string;
    weg?: string;
    text: string;
    alter?: string;
  }

  /**
   * Ein Strom statt vier Listen. Die Reihenfolge ist bewusst nach Art gruppiert und
   * nicht nach Relevanz: der Speicher liefert keine Bewertung, und eine erfundene
   * wäre schlechter als eine, die man versteht.
   */
  function zuStrom(e: {
    projects: Projekt[];
    notes: Notiz[];
    references: Referenz[];
    vaultEntries: TresorEintrag[];
  }): Treffer[] {
    return [
      ...e.projects.map((p) => ({
        art: 'projekt' as const,
        id: p.id,
        href: `#/projekt/${p.id}`,
        titel: p.titel,
        text: p.kurs || 'Noch kein Kurs gesetzt.',
        alter: alterInTagenText(p.zuletzt_beruehrt),
      })),
      ...e.notes.map((n) => ({
        art: 'notiz' as const,
        id: n.id,
        href: `#/projekt/${n.projekt_id}`,
        text: zeileMitTreffer(n.text, begriff),
        alter: alterInTagenText(n.ts),
      })),
      ...e.references.map((r) => ({
        art: 'referenz' as const,
        id: r.id,
        href: `#/projekt/${r.projekt_id}`,
        weg: PRUEFSTATUS_LABEL[r.pruefstatus],
        text: r.ziel,
      })),
      ...e.vaultEntries.map((v) => ({
        art: 'zugang' as const,
        id: v.id,
        href: '#/tresor',
        text: v.titel,
        weg: 'nur der Titel – Werte bleiben im Tresor',
      })),
    ];
  }

  let { query }: { query: string } = $props();

  const VERZOEGERUNG_MS = 180;

  let eingabe = $state('');
  let begriff = $state('');
  let inputEl: HTMLInputElement | undefined = $state();
  let timer: ReturnType<typeof setTimeout> | undefined;
  // Zuletzt aus der Adresse übernommener Begriff. Bewusst kein $state: dient nur dazu,
  // eigene Adressänderungen von fremden (Link, Zurück-Weg) zu unterscheiden.
  let ausAdresse: string | null = null;

  $effect(() => {
    if (query !== ausAdresse) {
      ausAdresse = query;
      eingabe = query;
      begriff = query.trim();
    }
  });

  // Beim Betreten der Seite steht der Fokus im Feld; die Suche ist ein Tastaturwerkzeug.
  $effect(() => {
    inputEl?.focus();
  });

  function beiEingabe() {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      begriff = eingabe.trim();
      ausAdresse = begriff;
      ersetze(begriff ? `#/suche?q=${encodeURIComponent(begriff)}` : '#/suche');
    }, VERZOEGERUNG_MS);
  }

  $effect(() => () => {
    if (timer) clearTimeout(timer);
  });

  let ergebnisPromise = $derived(begriff ? provider.search(begriff) : Promise.resolve(null));
</script>

<svelte:head><title>Suche · Lotse</title></svelte:head>

<!-- Kein „Suche“ über einem Suchfeld: das Feld ist die Seite. -->
<div class="suchkopf">
  <div class="suchfeld-gross">
    <svg viewBox="0 0 24 24" width="19" height="19" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
      <circle cx="11" cy="11" r="7" /><path d="m20 20-3.5-3.5" />
    </svg>
    <input
      bind:this={inputEl}
      bind:value={eingabe}
      oninput={beiEingabe}
      type="search"
      placeholder="Projekte, Logbuch, Referenzen, Tresor-Titel …"
      aria-label="Suche"
    />
  </div>
</div>

{#if !begriff}
  <p class="hinweis leer-hinweis">
    Tippen genügt – gesucht wird über alle Projekte, Logbücher, Referenzen und Tresor-Titel.
    <br />Tresor-<em>Werte</em> sind nie dabei, nur Titel.
  </p>
{:else}
  {#await ergebnisPromise}
    <p class="hinweis leer-hinweis">Suche läuft…</p>
  {:then ergebnis}
    {#if !ergebnis}
      <p class="hinweis leer-hinweis">Tippen genügt.</p>
    {:else}
      {@const strom = zuStrom(ergebnis)}
      {@const zahlen = {
        alles: strom.length,
        projekt: strom.filter((t) => t.art === 'projekt').length,
        notiz: strom.filter((t) => t.art === 'notiz').length,
        referenz: strom.filter((t) => t.art === 'referenz').length,
        zugang: strom.filter((t) => t.art === 'zugang').length,
      }}
      {@const sichtbar = art === 'alles' ? strom : strom.filter((t) => t.art === art)}

      {#if strom.length === 0}
        <p class="hinweis leer-hinweis">Keine Treffer für „{begriff}“.</p>
      {:else}
        <div class="such-flaeche">
          <nav class="arten" aria-label="Treffer filtern">
            {#each ARTEN as a (a.id)}
              {#if zahlen[a.id] > 0 || a.id === 'alles'}
                <button type="button" class:gewaehlt={art === a.id} onclick={() => (art = a.id)}>
                  <span>{a.name}</span>
                  <span class="art-zahl">{zahlen[a.id]}</span>
                </button>
              {/if}
            {/each}
            <p class="rail-notiz">
              Tresor-Werte sind nie dabei – nur Titel, und auch die nur hier, nie über die Schnittstelle für
              Assistenten.
            </p>
          </nav>

          <main class="treffer">
            {#each sichtbar as t (t.art + t.id)}
              <a class="treffer-zeile" href={t.href}>
                <span class="treffer-art art--{t.art}">{ART_NAME[t.art]}</span>
                <span class="treffer-leib">
                  {#if t.titel}<span class="treffer-titel">{t.titel}</span>{/if}
                  {#if t.weg}<span class="treffer-weg">{t.weg}</span>{/if}
                  <span class="treffer-text" class:mono={t.art === 'referenz'}>
                    {#each hervorheben(t.text, begriff) as teil, i (i)}
                      {#if teil.treffer}<mark>{teil.t}</mark>{:else}{teil.t}{/if}
                    {/each}
                  </span>
                </span>
                {#if t.alter}<span class="treffer-alter">{t.alter}</span>{/if}
              </a>
            {/each}
          </main>
        </div>
      {/if}
    {/if}
  {:catch fehler}
    <p class="hinweis fehler">Suche fehlgeschlagen: {fehler.message}</p>
  {/await}
{/if}

<style>
  .suchkopf {
    padding: 0.5rem 0 1.1rem;
  }
  /* Das Feld ist die Seite: ein Kasten in Lesegröße, unten eine Kante in Akzentfarbe,
     damit klar ist, dass hier getippt wird. */
  .suchfeld-gross {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    border: 1px solid var(--rahmen);
    border-bottom: 2px solid var(--akzent);
    border-radius: 0.75rem;
    background: var(--karten-hintergrund);
    padding: 0.8rem 1.1rem;
    box-shadow: var(--schatten);
    max-width: 48rem;
  }
  .suchfeld-gross svg {
    color: var(--text-gedaempft);
    flex: none;
  }
  .suchfeld-gross input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    padding: 0;
    font-size: 1.3rem;
    letter-spacing: -0.01em;
  }
  .suchfeld-gross input:focus {
    outline: none;
  }
  .suchfeld-gross:focus-within {
    border-color: var(--akzent);
  }

  .such-flaeche {
    display: grid;
    grid-template-columns: 11rem minmax(0, 1fr);
    gap: 1.75rem;
    align-items: start;
  }
  .arten {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    position: sticky;
    top: 1rem;
  }
  .arten button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border: none;
    background: transparent;
    color: var(--text-gedaempft);
    border-radius: 0.5rem;
    padding: 0.4rem 0.7rem;
    font-size: 0.85rem;
    text-align: left;
  }
  .arten button:hover {
    background: var(--flaeche-still);
    color: var(--fg);
  }
  .arten button.gewaehlt {
    background: var(--flaeche-still);
    color: var(--fg);
    font-weight: 600;
  }
  .art-zahl {
    margin-left: auto;
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
  }
  .rail-notiz {
    margin: 1rem 0.7rem 0;
    padding-top: 0.75rem;
    border-top: 1px solid var(--rahmen);
    font-size: 0.72rem;
    line-height: 1.5;
    color: var(--text-gedaempft);
  }
  @media (max-width: 60rem) {
    .such-flaeche {
      grid-template-columns: minmax(0, 1fr);
      gap: 0.75rem;
    }
    .arten {
      position: static;
      flex-direction: row;
      flex-wrap: wrap;
    }
    .rail-notiz {
      display: none;
    }
  }

  .treffer {
    min-width: 0;
    max-width: 52rem;
    display: flex;
    flex-direction: column;
  }
  .treffer-zeile {
    display: flex;
    align-items: flex-start;
    gap: 0.9rem;
    padding: 0.75rem 0.7rem;
    border-radius: 0.55rem;
    color: inherit;
    text-decoration: none;
    border-bottom: 1px solid var(--rahmen-still);
  }
  .treffer-zeile:hover {
    background: var(--flaeche-still);
    border-bottom-color: transparent;
  }
  .treffer-art {
    flex: none;
    width: 4.8rem;
    padding-top: 0.1rem;
    font-size: 0.66rem;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--text-gedaempft);
  }
  .treffer-art.art--projekt {
    color: var(--akzent);
  }
  .treffer-leib {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }
  .treffer-titel {
    font-weight: 600;
    font-size: 0.95rem;
  }
  .treffer-weg {
    font-size: 0.72rem;
    color: var(--text-gedaempft);
  }
  .treffer-text {
    font-size: 0.9rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }
  .treffer-text.mono {
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.8rem;
  }
  /* Der Treffer wird unterstrichen, nicht eingefärbt: eine Bernsteinfläche im
     Fließtext schreit lauter als die Zeile, in der sie steht. */
  mark {
    background: transparent;
    color: inherit;
    font-weight: 600;
    box-shadow: inset 0 -0.4em 0 color-mix(in srgb, var(--feuer) 45%, transparent);
    border-radius: 2px;
  }
  .treffer-alter {
    margin-left: auto;
    flex: none;
    font-size: 0.72rem;
    color: var(--text-gedaempft);
    padding-top: 0.15rem;
  }

  .leer-hinweis {
    margin: 0.5rem 0 0;
    line-height: 1.6;
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
</style>
