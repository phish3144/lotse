<script lang="ts">
  import { auffaelligkeit, type Auffaelligkeit } from '../lib/brief';
  import ProjektKarte from '../lib/components/ProjektKarte.svelte';
  import { provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';
  import { erfassung } from '../lib/erfassung.svelte';
  import { meldungen } from '../lib/meldung.svelte';
  import { VORLAGEN_LABEL } from '../lib/format';
  import NeuesProjekt from '../lib/components/NeuesProjekt.svelte';
  import type { Kandidat, Notiz, Projekt } from '../lib/data/types';

  interface Eintrag {
    projekt: Projekt;
    letzteNotiz?: Notiz;
    grad: Auffaelligkeit;
  }

  const GRAD_RANG: Record<Auffaelligkeit, number> = { ueberfaellig: 0, auffaellig: 1, ruhig: 2 };

  async function laden() {
    const [projekte, kandidaten] = await Promise.all([provider.listProjects(), provider.listCandidates()]);
    const eintraege: Eintrag[] = await Promise.all(
      projekte.map(async (projekt) => {
        const notizen = await provider.listNotes(projekt.id);
        return { projekt, letzteNotiz: notizen[0], grad: auffaelligkeit(projekt, notizen) };
      }),
    );
    return { eintraege, kandidaten };
  }

  let datenPromise = $derived.by(() => {
    datenVersion.wert;
    return laden();
  });

  let bestaetigtWirdGerade: string | undefined = $state(undefined);
  let neuesProjektOffen = $state(false);
  let fehlerText: string | null = $state(null);

  async function bestaetigen(kandidat: Kandidat) {
    bestaetigtWirdGerade = kandidat.id;
    fehlerText = null;
    try {
      const projekt = await provider.confirmCandidate(kandidat.id);
      datenVersion.bump();
      meldungen.zeigen(`„${projekt.titel}“ übernommen.`);
    } catch (e) {
      fehlerText = e instanceof Error ? e.message : String(e);
      meldungen.fehler(e);
    } finally {
      bestaetigtWirdGerade = undefined;
    }
  }

  async function verwerfen(kandidat: Kandidat) {
    bestaetigtWirdGerade = kandidat.id;
    fehlerText = null;
    try {
      await provider.rejectCandidate(kandidat.id);
      datenVersion.bump();
      meldungen.zeigen(`„${kandidat.titel_vorschlag}“ verworfen.`);
    } catch (e) {
      fehlerText = e instanceof Error ? e.message : String(e);
      meldungen.fehler(e);
    } finally {
      bestaetigtWirdGerade = undefined;
    }
  }

  function sortiertNachAuffaelligkeit(liste: Eintrag[]): Eintrag[] {
    return [...liste].sort(
      (a, b) => GRAD_RANG[a.grad] - GRAD_RANG[b.grad] || b.projekt.zuletzt_beruehrt.localeCompare(a.projekt.zuletzt_beruehrt),
    );
  }
</script>

<svelte:head><title>Hafen · Lotse</title></svelte:head>

<NeuesProjekt bind:offen={neuesProjektOffen} />

<div class="seiten-kopf">
  <h1>Hafen</h1>
  <div class="kopf-aktionen">
    <button type="button" onclick={() => erfassung.oeffnen()}>Erfassen</button>
    <button type="button" class="primaer" onclick={() => (neuesProjektOffen = true)}>Neues Projekt</button>
  </div>
</div>

{#if fehlerText}
  <p class="hinweis fehler">{fehlerText}</p>
{/if}

{#await datenPromise}
  <p class="hinweis">Lade Hafen…</p>
{:then { eintraege, kandidaten }}
  {@const heuteWichtig = sortiertNachAuffaelligkeit(
    eintraege.filter((e) => e.grad !== 'ruhig' && e.projekt.status !== 'abgeschlossen' && e.projekt.status !== 'eingemottet'),
  ).slice(0, 3)}
  <!-- Was oben schon Aufmerksamkeit bekommt, wird unten nicht wiederholt. -->
  {@const obenGezeigt = new Set(heuteWichtig.map((e) => e.projekt.id))}
  {@const uebrig = eintraege.filter((e) => !obenGezeigt.has(e.projekt.id))}
  {@const aufSee = sortiertNachAuffaelligkeit(uebrig.filter((e) => e.projekt.status === 'aktiv'))}
  {@const vorAnker = sortiertNachAuffaelligkeit(uebrig.filter((e) => e.projekt.status === 'pausiert' || e.projekt.status === 'wartet'))}
  {@const ideen = sortiertNachAuffaelligkeit(uebrig.filter((e) => e.projekt.status === 'idee'))}

  <!-- Zwei Bahnen: die Vorhaben links, die Hafeneinfahrt rechts. Sie war vorher
       zwischen „Heute wichtig“ und „Auf See“ eingeschoben und hat den Blick auf die
       Projekte zerschnitten. Unter 64rem fällt sie wieder darunter. -->
  <div class="hafen-flaeche">
    <div class="hafen-haupt">
    {#if heuteWichtig.length > 0}
      <section aria-labelledby="heute-wichtig-titel">
        <h2 id="heute-wichtig-titel">Heute wichtig</h2>
        <div class="karten-raster">
          {#each heuteWichtig as e (e.projekt.id)}
            <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
          {/each}
        </div>
      </section>
    {/if}

    {#if eintraege.length === 0 && kandidaten.length === 0}
        <!-- Ein leerer Hafen ist der erste Eindruck, kein Fehlerzustand: statt „keine
           Daten“ steht hier, wofür das Ganze gut ist und wie man hineinkommt. -->
      <section class="leer">
        <svg class="leer-bild" viewBox="0 0 240 120" aria-hidden="true">
          <g fill="none" stroke="currentColor" stroke-width="1.6" opacity="0.5">
            <path d="M-10 84 C 40 72, 72 96, 120 84 S 200 70, 250 88" />
            <path d="M-10 100 C 40 88, 72 112, 120 100 S 200 86, 250 104" />
          </g>
          <g fill="none" stroke="currentColor" stroke-width="1.8" opacity="0.85">
            <path d="M120 24 v52" />
            <path d="M120 30 l34 16 -34 16 z" fill="currentColor" opacity="0.14" />
            <path d="M120 76 h-22 a8 8 0 0 0 0 8 h44 a8 8 0 0 0 0-8 z" opacity="0.5" />
          </g>
        </svg>

        <h2 class="serife">Noch nichts an Bord.</h2>
        <p class="leer-text">
          Lotse merkt sich, was du getan hast und was als Nächstes dran ist – für alles, was länger dauert als ein
          Nachmittag. Drei Wege hinein:
        </p>

        <ol class="wege">
          <li>
            <span class="weg-zahl">1</span>
            <div>
              <strong>Ordner durchsuchen lassen.</strong>
              <span>
                Lotse sucht nach Marken wie <code>.git</code>, <code>Cargo.toml</code> oder einer README und schlägt
                vor, was es findet. Übernommen wird nichts von allein, und Dateiinhalte liest es dabei nicht.
              </span>
              <a class="weg-knopf" href="#/einstellungen/ordner">Ordner wählen …</a>
            </div>
          </li>
          <li>
            <span class="weg-zahl">2</span>
            <div>
              <strong>Ein Vorhaben von Hand anlegen.</strong>
              <span>
                Titel und ein Satz dazu, worum es geht. Das ist der Kurs – die eine Zeile, die dir in drei Monaten
                sagt, was du eigentlich wolltest.
              </span>
              <button class="weg-knopf" type="button" onclick={() => (neuesProjektOffen = true)}>
                Neues Projekt …
              </button>
            </div>
          </li>
          <li>
            <span class="weg-zahl">3</span>
            <div>
              <strong>Einfach anfangen zu schreiben.</strong>
              <span>
                <kbd>Strg</kbd>+<kbd>K</kbd> von überall. Ohne Zuordnung landet der Gedanke im Postkorb und wartet
                dort, bis du weißt, wohin er gehört.
              </span>
            </div>
          </li>
        </ol>

        <p class="leer-fuss">
          Alles bleibt auf diesem Rechner, bis du den Abgleich einrichtest. Und wenn Lotse morgen weg ist, kommst du
          trotzdem an alles: ein Befehl legt jedes Vorhaben als Markdown ab.
        </p>
      </section>
    {/if}

    <section aria-labelledby="auf-see-titel">
      <h2 id="auf-see-titel">Auf See</h2>
      {#if aufSee.length === 0}
        <p class="hinweis">Keine aktiven Projekte.</p>
      {:else}
        <div class="karten-raster">
          {#each aufSee as e (e.projekt.id)}
            <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
          {/each}
        </div>
      {/if}
    </section>

    {#if vorAnker.length > 0}
      <details class="gruppe">
        <summary><h2>Vor Anker ({vorAnker.length})</h2></summary>
        <div class="karten-raster">
          {#each vorAnker as e (e.projekt.id)}
            <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
          {/each}
        </div>
      </details>
    {/if}

    {#if ideen.length > 0}
          <section aria-labelledby="ideen-titel">
            <h2 id="ideen-titel">Ideen</h2>
            <div class="karten-raster">
              {#each ideen as e (e.projekt.id)}
                <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
              {/each}
            </div>
          </section>
        {/if}
    </div>

    {#if kandidaten.length > 0}
      <aside class="hafen-neben">
        <section class="einfahrt" aria-labelledby="hafeneinfahrt-titel">
          <h2 id="hafeneinfahrt-titel">Hafeneinfahrt</h2>
          <p class="hinweis">Beim Beobachten gefunden. Nichts davon ist übernommen.</p>
          <ul class="kandidaten-liste">
            {#each kandidaten as kandidat (kandidat.id)}
              <li class="kandidat">
                <div class="kandidat-text">
                  <strong>{kandidat.titel_vorschlag}</strong>
                  <span class="kandidat-details">
                    {kandidat.pfad} · {VORLAGEN_LABEL[kandidat.vorlage_vorschlag]} · {kandidat.erkennungsmarke}
                  </span>
                </div>
                <div class="kandidat-aktionen">
                  <button type="button" onclick={() => verwerfen(kandidat)} disabled={bestaetigtWirdGerade === kandidat.id}>
                    Verwerfen
                  </button>
                  <button
                    type="button"
                    class="uebernehmen"
                    onclick={() => bestaetigen(kandidat)}
                    disabled={bestaetigtWirdGerade === kandidat.id}
                  >
                    {bestaetigtWirdGerade === kandidat.id ? 'Moment …' : 'Übernehmen'}
                  </button>
                </div>
              </li>
            {/each}
          </ul>
        </section>
      </aside>
    {/if}
  </div>
{:catch fehler}
  <p class="hinweis fehler">Hafen konnte nicht geladen werden: {fehler.message}</p>
{/await}

<style>
  .seiten-kopf {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    margin-bottom: 1.5rem;
  }
  .seiten-kopf h1 {
    margin: 0;
    font-size: 1.7rem;
    font-family: ui-serif, Georgia, 'Iowan Old Style', 'Times New Roman', serif;
    font-weight: 600;
  }
  .kopf-aktionen {
    display: flex;
    gap: 0.5rem;
  }
  .kopf-aktionen button:not(.primaer),
  .kandidat-aktionen button:not(.primaer) {
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: inherit;
    border-radius: 0.5rem;
    padding: 0.4rem 0.8rem;
  }
  .kopf-aktionen button:hover,
  .kandidat-aktionen button:hover {
    border-color: var(--akzent);
  }
  .kandidat-aktionen {
    display: flex;
    gap: 0.4rem;
    flex: none;
  }
  .leer {
    max-width: 40rem;
    margin: 1rem auto 0;
    text-align: center;
  }
  .leer-bild {
    width: 15rem;
    height: 7.5rem;
    color: var(--akzent);
    opacity: 0.7;
  }
  .leer h2 {
    margin: 0.5rem 0 0.6rem;
    font-size: 1.6rem;
    font-weight: 600;
    text-transform: none;
    letter-spacing: -0.015em;
    color: var(--fg);
  }
  .leer-text {
    margin: 0 auto 1.5rem;
    max-width: 32rem;
    line-height: 1.6;
    color: var(--text-gedaempft);
  }
  .wege {
    list-style: none;
    margin: 0;
    padding: 0;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .wege li {
    display: flex;
    gap: 0.85rem;
    align-items: flex-start;
    background: var(--karten-hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.7rem;
    padding: 0.85rem 1rem;
    box-shadow: var(--schatten);
  }
  .wege li > div {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    align-items: flex-start;
  }
  .weg-zahl {
    flex: none;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 50%;
    background: var(--flaeche-still);
    color: var(--text-gedaempft);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 700;
  }
  .wege strong {
    font-size: 0.95rem;
  }
  .wege span {
    font-size: 0.85rem;
    line-height: 1.55;
    color: var(--text-gedaempft);
  }
  .weg-knopf {
    margin-top: 0.35rem;
    display: inline-block;
    border: 1px solid var(--rahmen);
    background: var(--hintergrund);
    color: inherit;
    text-decoration: none;
    border-radius: 0.5rem;
    padding: 0.35rem 0.75rem;
    font-size: 0.85rem;
  }
  .weg-knopf:hover {
    border-color: var(--akzent);
  }
  .leer-fuss {
    margin: 1.5rem auto 0;
    max-width: 34rem;
    font-size: 0.8rem;
    line-height: 1.6;
    color: var(--text-gedaempft);
  }
  kbd {
    font-family: inherit;
    font-size: 0.85em;
    border: 1px solid var(--rahmen);
    border-radius: 0.25rem;
    padding: 0.05em 0.35em;
  }

  /* Zwei Bahnen. Die Hafeneinfahrt hat vorher die Projektblöcke zerschnitten. */
  .hafen-flaeche {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 22rem;
    gap: 1.75rem;
    align-items: start;
  }
  .hafen-haupt {
    min-width: 0;
  }
  .hafen-neben {
    min-width: 0;
    position: sticky;
    top: 1rem;
  }
  @media (max-width: 64rem) {
    .hafen-flaeche {
      grid-template-columns: minmax(0, 1fr);
    }
    .hafen-neben {
      position: static;
    }
  }

  section,
  .gruppe {
    margin-bottom: 1.75rem;
  }
  h2 {
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-gedaempft);
    margin: 0 0 0.7rem;
  }
  .gruppe summary {
    cursor: pointer;
    list-style: none;
  }
  .gruppe summary::-webkit-details-marker {
    display: none;
  }
  .gruppe summary h2 {
    display: inline-block;
    margin: 0 0 0.75rem;
  }
  .gruppe summary::before {
    content: '▸ ';
    color: var(--text-gedaempft);
  }
  .gruppe summary:hover h2 {
    color: var(--fg);
  }
  .gruppe[open] summary::before {
    content: '▾ ';
  }
  .karten-raster {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(15rem, 1fr));
    gap: 0.7rem;
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
  .kandidaten-liste {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  /* Die Hafeneinfahrt ist die einzige Karte mit Bernstein am Rand: hier wartet
     etwas auf eine Entscheidung. */
  .einfahrt {
    border: 1px solid var(--rahmen);
    border-left: 3px solid var(--feuer);
    border-radius: 0.7rem;
    background: var(--karten-hintergrund);
    padding: 0.8rem 0.9rem;
    box-shadow: var(--schatten);
    margin-bottom: 0;
  }
  .einfahrt .hinweis {
    font-size: 0.8rem;
    margin: 0 0 0.7rem;
  }
  .kandidat {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.7rem 0;
    border-top: 1px solid var(--rahmen-still);
  }
  .kandidat:first-child {
    border-top: none;
    padding-top: 0;
  }
  .kandidat-text {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
  }
  /* Kein zweites Bernstein: in einer Liste wiederholt sich die Handlung, und ein
     Leuchtfeuer, das dreimal blinkt, ist keins mehr. */
  .kandidat-aktionen .uebernehmen {
    border: 1px solid var(--akzent);
    background: var(--karten-hintergrund);
    color: var(--akzent);
    font-weight: 600;
    border-radius: 0.5rem;
    padding: 0.4rem 0.8rem;
  }
  .kandidat-aktionen .uebernehmen:disabled {
    opacity: 0.5;
  }
  .kandidat-details {
    font-size: 0.78rem;
    color: var(--text-gedaempft);
    overflow-wrap: anywhere;
  }
</style>
