<script lang="ts">
  import AuffaelligkeitPunkt from '../lib/components/AuffaelligkeitPunkt.svelte';
  import NoteText from '../lib/components/NoteText.svelte';
  import { provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';
  import { auffaelligkeit, type Auffaelligkeit } from '../lib/brief';
  import type { Notiz } from '../lib/data/types';
  import { alterInTagenText } from '../lib/format';
  import { meldungen } from '../lib/meldung.svelte';

  /**
   * Gebündelt nach Vorhaben, nicht nach Datum: ein Faden ohne sein Projekt ist eine
   * Zeile ohne Zusammenhang. Die Reihenfolge der Bündel folgt der Auffälligkeit des
   * Projekts – was lange ruht, steht oben, weil dort am ehesten etwas hängen bleibt.
   */
  async function laden() {
    const [faeden, projekte] = await Promise.all([provider.listOpenThreads(), provider.listProjects()]);
    const nachId = new Map(projekte.map((p) => [p.id, p] as const));

    const buendel = new Map<string, { titel: string; grad: Auffaelligkeit; beruehrt?: string; faeden: Notiz[] }>();
    for (const faden of faeden) {
      let b = buendel.get(faden.projekt_id);
      if (!b) {
        const projekt = nachId.get(faden.projekt_id);
        b = {
          titel: projekt?.titel ?? faden.projekt_id,
          // Ohne Projekt (kann bei einem halb abgeglichenen Bestand vorkommen) lieber
          // „ruhig“ behaupten als eine Warnung erfinden.
          grad: projekt ? auffaelligkeit(projekt, [], new Date()) : 'ruhig',
          beruehrt: projekt?.zuletzt_beruehrt,
          faeden: [],
        };
        buendel.set(faden.projekt_id, b);
      }
      b.faeden.push(faden);
    }

    const rang: Record<Auffaelligkeit, number> = { ueberfaellig: 0, auffaellig: 1, ruhig: 2 };
    const gruppen = [...buendel.entries()]
      .map(([projektId, b]) => ({ projektId, ...b }))
      .sort((a, b) => rang[a.grad] - rang[b.grad] || a.titel.localeCompare(b.titel, 'de'));

    const ohneFaden = projekte
      .filter((p) => p.status === 'aktiv' && !buendel.has(p.id))
      .map((p) => ({ id: p.id, titel: p.titel }));

    const aeltesteZuerst = [...faeden].sort((a, b) => a.ts.localeCompare(b.ts)).slice(0, 3);

    return { gruppen, ohneFaden, aeltesteZuerst, anzahl: faeden.length };
  }

  let datenPromise = $derived.by(() => {
    datenVersion.wert;
    return laden();
  });

  let fehlerText: string | null = $state(null);
  let laeuft: string | null = $state(null);

  async function erledigen(notizId: string) {
    laeuft = notizId;
    fehlerText = null;
    try {
      await provider.completeThread(notizId);
      datenVersion.bump();
      meldungen.zeigen('Faden abgehakt.');
    } catch (e) {
      fehlerText = e instanceof Error ? e.message : String(e);
    } finally {
      laeuft = null;
    }
  }
</script>

<svelte:head><title>Offene Punkte · Lotse</title></svelte:head>

{#await datenPromise}
  <div class="seiten-kopf"><h1 class="serife">Offene Punkte</h1></div>
  <p class="hinweis">Lade offene Punkte…</p>
{:then { gruppen, ohneFaden, aeltesteZuerst, anzahl }}
  <div class="seiten-kopf">
    <h1 class="serife">Offene Punkte</h1>
    <span class="kopf-meta">
      {#if anzahl === 0}
        nichts offen
      {:else}
        {anzahl}
        {anzahl === 1 ? 'Faden' : 'Fäden'} über {gruppen.length}
        {gruppen.length === 1 ? 'Vorhaben' : 'Vorhaben'}
      {/if}
    </span>
  </div>

  {#if fehlerText}
    <p class="hinweis fehler">{fehlerText}</p>
  {/if}

  <div class="punkte-flaeche">
    <main class="punkte-haupt">
      {#if anzahl === 0}
        <p class="hinweis">
          Keine offenen Fäden. Beim Abschließen fragt Lotse nach einer Sache fürs nächste Mal – so bleibt selten
          nichts übrig.
        </p>
      {:else}
        {#each gruppen as gruppe (gruppe.projektId)}
          <section class="buendel">
            <div class="buendel-kopf">
              <AuffaelligkeitPunkt wert={gruppe.grad} />
              <a class="buendel-name" href={`#/projekt/${gruppe.projektId}`}>{gruppe.titel}</a>
              {#if gruppe.beruehrt}
                <span class="buendel-alter">{alterInTagenText(gruppe.beruehrt)} berührt</span>
              {/if}
            </div>
            <ul class="faeden">
              {#each gruppe.faeden as faden (faden.id)}
                <li class="faden">
                  <button
                    type="button"
                    class="haken"
                    title="Faden abhaken"
                    aria-label="Faden abhaken"
                    onclick={() => erledigen(faden.id)}
                    disabled={laeuft === faden.id}
                  >
                    ✓
                  </button>
                  <div class="faden-text"><NoteText text={faden.text} /></div>
                  <span class="faden-alter">{alterInTagenText(faden.ts)}</span>
                </li>
              {/each}
            </ul>
          </section>
        {/each}
      {/if}
    </main>

    <aside class="punkte-neben">
      <section class="karte">
        <h2>Was das hier nicht ist</h2>
        <p>
          Keine Aufgabenliste mit Prioritäten, Zuweisungen und Fristen. Ein offener Faden ist eine Frage, die beim
          nächsten Mal im Weg steht – mehr nicht. Wer Tickets braucht, braucht ein Ticketsystem.
        </p>
      </section>

      {#if aeltesteZuerst.length > 0}
        <section class="karte">
          <h2>Am längsten offen</h2>
          <ul class="mini-liste">
            {#each aeltesteZuerst as faden (faden.id)}
              <li>
                <span class="mini-alter">{alterInTagenText(faden.ts)}</span>
                <a href={`#/projekt/${faden.projekt_id}`}>{faden.text.split('\n')[0]}</a>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if ohneFaden.length > 0}
        <section class="karte">
          <h2>Ohne Faden <span class="zahl">{ohneFaden.length}</span></h2>
          <p>
            Diese aktiven Vorhaben haben keinen offenen Faden. Das ist in Ordnung – nur weiß beim nächsten Mal
            niemand, wo es weitergeht.
          </p>
          <ul class="mini-liste">
            {#each ohneFaden as p (p.id)}
              <li><a href={`#/projekt/${p.id}`}>{p.titel}</a></li>
            {/each}
          </ul>
        </section>
      {/if}
    </aside>
  </div>
{:catch fehler}
  <p class="hinweis fehler">Offene Punkte konnten nicht geladen werden: {fehler.message}</p>
{/await}

<style>
  .seiten-kopf {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    flex-wrap: wrap;
    margin-bottom: 1.25rem;
  }
  .seiten-kopf h1 {
    margin: 0;
    font-size: 1.7rem;
    font-weight: 600;
  }
  .kopf-meta {
    font-size: 0.8rem;
    color: var(--text-gedaempft);
  }

  .punkte-flaeche {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 22rem;
    gap: 1.75rem;
    align-items: start;
  }
  .punkte-haupt {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }
  .punkte-neben {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    position: sticky;
    top: 1rem;
  }
  @media (max-width: 64rem) {
    .punkte-flaeche {
      grid-template-columns: minmax(0, 1fr);
    }
    .punkte-neben {
      position: static;
    }
  }

  /* Kein Rahmen um jeden Faden: die Bündelüberschrift trägt die Zusammengehörigkeit,
     eine Linie darunter reicht. Vorher war jede Zeile eine eigene Kiste. */
  .buendel-kopf {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding-bottom: 0.45rem;
    border-bottom: 1px solid var(--rahmen);
  }
  .buendel-name {
    font-weight: 600;
    font-size: 0.95rem;
    color: inherit;
    text-decoration: none;
  }
  .buendel-name:hover {
    color: var(--akzent);
  }
  .buendel-alter {
    margin-left: auto;
    font-size: 0.75rem;
    color: var(--text-gedaempft);
  }

  .faeden {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .faden {
    display: flex;
    align-items: flex-start;
    gap: 0.7rem;
    padding: 0.55rem 0.4rem 0.55rem 0.15rem;
    border-radius: 0.5rem;
    line-height: 1.45;
  }
  .faden:hover {
    background: var(--flaeche-still);
  }
  .faden-text {
    flex: 1 1 0;
    min-width: 0;
  }
  .faden-alter {
    flex: none;
    font-size: 0.75rem;
    color: var(--text-gedaempft);
    white-space: nowrap;
    padding-top: 0.1rem;
  }
  .haken {
    flex: none;
    width: 1.15rem;
    height: 1.15rem;
    margin-top: 0.1rem;
    border: 1.5px solid var(--rahmen);
    border-radius: 0.35rem;
    background: var(--karten-hintergrund);
    color: transparent;
    font-size: 0.75rem;
    line-height: 1;
    padding: 0;
  }
  .haken:hover:not(:disabled) {
    border-color: var(--farbe-ruhig);
    color: var(--farbe-ruhig);
  }
  .haken:disabled {
    opacity: 0.5;
  }

  .karte {
    border: 1px solid var(--rahmen);
    border-radius: 0.7rem;
    background: var(--karten-hintergrund);
    padding: 0.75rem 0.85rem;
    box-shadow: var(--schatten);
  }
  .karte h2 {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-gedaempft);
    margin: 0 0 0.5rem;
  }
  .zahl {
    font-size: 0.7rem;
    background: var(--flaeche-still);
    border-radius: 999px;
    padding: 0.05rem 0.4rem;
    letter-spacing: 0;
  }
  .karte p {
    margin: 0 0 0.6rem;
    font-size: 0.8rem;
    line-height: 1.5;
    color: var(--text-gedaempft);
  }
  .karte p:last-child {
    margin-bottom: 0;
  }
  .mini-liste {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font-size: 0.83rem;
  }
  .mini-liste li {
    display: flex;
    gap: 0.6rem;
    align-items: baseline;
  }
  .mini-liste a {
    color: inherit;
    text-decoration: none;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mini-liste a:hover {
    color: var(--akzent);
  }
  .mini-alter {
    flex: none;
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-gedaempft);
    white-space: nowrap;
  }

  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
</style>
