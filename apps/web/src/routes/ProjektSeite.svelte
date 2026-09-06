<script lang="ts">
  import { computeBrief, sollBriefZeigen } from '../lib/brief';
  import NoteText from '../lib/components/NoteText.svelte';
  import { provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';
  import { alterInTagenText, datumText, PRUEFSTATUS_LABEL, QUELLE_LABEL, STATUS_LABEL, tageText } from '../lib/format';
  import type { Notiz, ProjektStatus } from '../lib/data/types';

  let { id }: { id: string } = $props();

  const STATUS_OPTIONEN: ProjektStatus[] = ['idee', 'aktiv', 'pausiert', 'wartet', 'abgeschlossen', 'eingemottet'];

  async function laden(projektId: string) {
    const [projekt, notizen, referenzen, zugaenge] = await Promise.all([
      provider.getProject(projektId),
      provider.listNotes(projektId),
      provider.listReferences(projektId),
      provider.listVaultEntries(projektId),
    ]);
    return { projekt, notizen, referenzen, zugaenge };
  }

  let datenPromise = $derived.by(() => {
    datenVersion.wert;
    return laden(id);
  });

  // Statuswechsel: braucht bei pausiert/wartet eine Übergabenotiz (CONCEPT.md Abschnitt 4).
  let menuOffen = $state(false);
  let ausstehenderStatus: ProjektStatus | null = $state(null);
  let uebergabeText = $state('');
  let wirdGespeichert = $state(false);
  let fehlerText: string | null = $state(null);

  function brauchtUebergabe(status: ProjektStatus): boolean {
    return status === 'pausiert' || status === 'wartet';
  }

  function statusWaehlen(status: ProjektStatus, offeneFaeden: Notiz[]) {
    menuOffen = false;
    fehlerText = null;
    if (brauchtUebergabe(status)) {
      ausstehenderStatus = status;
      uebergabeText = offeneFaeden.length > 0 ? offeneFaeden.map((f) => `- ${f.text}`).join('\n') : '';
    } else {
      void speichereStatus(status);
    }
  }

  function defaultUebergabeUebernehmen() {
    uebergabeText = 'Nichts Neues, siehe letzte Notiz.';
  }

  async function speichereStatus(status: ProjektStatus, text?: string) {
    wirdGespeichert = true;
    fehlerText = null;
    try {
      await provider.setStatus(id, status, text);
      ausstehenderStatus = null;
      uebergabeText = '';
      datenVersion.bump();
    } catch (e) {
      fehlerText = e instanceof Error ? e.message : String(e);
    } finally {
      wirdGespeichert = false;
    }
  }

  function uebergabeAbschicken() {
    if (!ausstehenderStatus) return;
    if (!uebergabeText.trim()) {
      fehlerText = 'Übergabenotiz ist beim Wechsel zu pausiert/wartet verpflichtend.';
      return;
    }
    void speichereStatus(ausstehenderStatus, uebergabeText);
  }

  function uebergabeAbbrechen() {
    ausstehenderStatus = null;
    uebergabeText = '';
    fehlerText = null;
  }

  let seitentitel = $state('Lotse');
  $effect(() => {
    datenPromise
      .then((d) => {
        seitentitel = d.projekt ? `${d.projekt.titel} · Lotse` : 'Lotse';
      })
      .catch(() => {
        seitentitel = 'Lotse';
      });
  });
</script>

<svelte:head><title>{seitentitel}</title></svelte:head>

{#await datenPromise}
  <p class="hinweis">Lade Projekt…</p>
{:then { projekt, notizen, referenzen, zugaenge }}
  {#if !projekt}
    <p class="hinweis fehler">Projekt nicht gefunden.</p>
  {:else}
    {@const offeneFaeden = notizen.filter((n) => n.art === 'offen' && !n.erledigt_am)}
    {@const brief = sollBriefZeigen(projekt, notizen) ? computeBrief(projekt, notizen) : null}

    {#if brief}
      <section class="brief-karte" aria-labelledby="brief-titel">
        <h2 id="brief-titel">Wo war ich</h2>
        <p class="brief-zeile">Letzter Kontakt: {tageText(brief.tageSeitLetztemKontakt)}.</p>
        {#if brief.letzteUebergabe}
          <div class="brief-uebergabe">
            <strong>Letzte Übergabe:</strong>
            <NoteText text={brief.letzteUebergabe.text} />
          </div>
        {/if}
        {#if brief.offeneFaeden.length > 0}
          <p class="brief-zeile"><strong>Offene Fäden:</strong> {brief.offeneFaeden.length}</p>
        {/if}
        {#if Object.keys(brief.aktivitaetSeitLetztemBesuch).length > 0}
          <div class="brief-aktivitaet">
            <strong>Aktivität seit dem letzten Besuch:</strong>
            <ul>
              {#each Object.entries(brief.aktivitaetSeitLetztemBesuch) as [quelle, eintraege]}
                <li>{QUELLE_LABEL[quelle as keyof typeof QUELLE_LABEL]}: {eintraege?.length} Einträge</li>
              {/each}
            </ul>
          </div>
        {/if}
      </section>
    {/if}

    <header class="kopf">
      <h1>{projekt.titel}</h1>
      <div class="status-bereich">
        <button type="button" class="status-chip status--{projekt.status}" onclick={() => (menuOffen = !menuOffen)}>
          {STATUS_LABEL[projekt.status]} ▾
        </button>
        {#if menuOffen}
          <ul class="status-menu">
            {#each STATUS_OPTIONEN.filter((s) => s !== projekt.status) as option (option)}
              <li>
                <button type="button" onclick={() => statusWaehlen(option, offeneFaeden)}>{STATUS_LABEL[option]}</button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </header>

    <p class="kurs">{projekt.kurs || 'Noch kein Kurs gesetzt.'}</p>

    {#if ausstehenderStatus}
      <section class="uebergabe-formular" aria-labelledby="uebergabe-titel">
        <h2 id="uebergabe-titel">Übergabenotiz für Wechsel zu „{STATUS_LABEL[ausstehenderStatus]}“</h2>
        <p class="hinweis">Pflicht, aber billig: kurz festhalten, was der nächste Einstieg wissen muss.</p>
        <textarea rows="4" bind:value={uebergabeText} placeholder="Was ist der Stand? Was ist als Nächstes dran?"></textarea>
        <div class="uebergabe-aktionen">
          <button type="button" onclick={defaultUebergabeUebernehmen}>Nichts Neues, siehe letzte Notiz</button>
          <button type="button" onclick={uebergabeAbschicken} disabled={wirdGespeichert}>Speichern</button>
          <button type="button" onclick={uebergabeAbbrechen}>Abbrechen</button>
        </div>
      </section>
    {/if}
    {#if fehlerText}
      <p class="hinweis fehler">{fehlerText}</p>
    {/if}

    <section aria-labelledby="offene-faeden-titel">
      <h2 id="offene-faeden-titel">Offene Fäden</h2>
      {#if offeneFaeden.length === 0}
        <p class="hinweis">Keine offenen Fäden.</p>
      {:else}
        <ul class="einfache-liste">
          {#each offeneFaeden as faden (faden.id)}
            <li><NoteText text={faden.text} /> <span class="alter">{alterInTagenText(faden.ts)}</span></li>
          {/each}
        </ul>
      {/if}
    </section>

    <section aria-labelledby="referenzen-titel">
      <h2 id="referenzen-titel">Referenzen</h2>
      {#if referenzen.length === 0}
        <p class="hinweis">Keine Referenzen.</p>
      {:else}
        <ul class="einfache-liste">
          {#each referenzen as ref (ref.id)}
            <li>
              <span class="referenz-typ">{ref.typ}</span>
              <span class="referenz-ziel">{ref.ziel}</span>
              <span class="badge badge--{ref.pruefstatus}">{PRUEFSTATUS_LABEL[ref.pruefstatus]}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section aria-labelledby="zugaenge-titel">
      <h2 id="zugaenge-titel">Zugänge</h2>
      {#if zugaenge.length === 0}
        <p class="hinweis">Keine Zugänge hinterlegt.</p>
      {:else}
        <ul class="einfache-liste">
          {#each zugaenge as eintrag (eintrag.id)}
            <li>
              <span>{eintrag.titel}</span>
              <span class="badge badge--stufe-{eintrag.stufe}">{eintrag.stufe === 'ueberall' ? 'überall' : 'nur Desktop'}</span>
              <span class="hinweis">({eintrag.felder.length} Feld{eintrag.felder.length === 1 ? '' : 'er'}, Werte verdeckt)</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section aria-labelledby="logbuch-titel">
      <h2 id="logbuch-titel">Logbuch</h2>
      {#if notizen.length === 0}
        <p class="hinweis">Noch keine Einträge.</p>
      {:else}
        <ul class="logbuch">
          {#each notizen as notiz (notiz.id)}
            <li>
              <div class="logbuch-kopf">
                <span class="badge badge--quelle">{QUELLE_LABEL[notiz.quelle]}</span>
                <span class="hinweis">{notiz.art}</span>
                <span class="alter" title={datumText(notiz.ts)}>{alterInTagenText(notiz.ts)}</span>
              </div>
              <NoteText text={notiz.text} />
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
{:catch fehler}
  <p class="hinweis fehler">Projekt konnte nicht geladen werden: {fehler.message}</p>
{/await}

<style>
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
  .brief-karte {
    border: 1px solid var(--akzent);
    border-radius: 0.6rem;
    padding: 1rem 1.2rem;
    margin-bottom: 1.5rem;
    background: var(--karten-hintergrund);
  }
  .brief-karte h2 {
    margin-top: 0;
  }
  .brief-zeile {
    margin: 0.4rem 0;
  }
  .brief-uebergabe {
    margin: 0.6rem 0;
    padding: 0.6rem 0.8rem;
    background: var(--hintergrund);
    border-radius: 0.4rem;
  }
  .brief-aktivitaet ul {
    margin: 0.3rem 0 0;
    padding-left: 1.2rem;
  }

  .kopf {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .kopf h1 {
    margin: 0;
  }
  .kurs {
    font-size: 1.05rem;
    margin: 0.5rem 0 1.5rem;
  }

  .status-bereich {
    position: relative;
  }
  .status-chip {
    border: 1px solid var(--rahmen);
    border-radius: 999px;
    padding: 0.3rem 0.9rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    background: var(--karten-hintergrund);
    color: inherit;
  }
  .status--pausiert,
  .status--wartet {
    color: var(--farbe-auffaellig);
    border-color: var(--farbe-auffaellig);
  }
  .status--abgeschlossen,
  .status--eingemottet {
    color: var(--text-gedaempft);
  }
  .status--aktiv {
    color: var(--akzent);
    border-color: var(--akzent);
  }
  .status-menu {
    position: absolute;
    right: 0;
    top: calc(100% + 0.3rem);
    list-style: none;
    margin: 0;
    padding: 0.3rem;
    background: var(--karten-hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    z-index: 10;
    min-width: 10rem;
  }
  .status-menu li button {
    display: block;
    width: 100%;
    text-align: left;
    padding: 0.4rem 0.6rem;
    border: none;
    background: none;
    color: inherit;
    cursor: pointer;
    border-radius: 0.3rem;
  }
  .status-menu li button:hover {
    background: var(--hintergrund);
  }

  .uebergabe-formular {
    border: 1px solid var(--farbe-auffaellig);
    border-radius: 0.6rem;
    padding: 1rem 1.2rem;
    margin-bottom: 1.5rem;
  }
  .uebergabe-formular textarea {
    width: 100%;
    box-sizing: border-box;
    font: inherit;
    margin: 0.5rem 0;
  }
  .uebergabe-aktionen {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  section {
    margin-bottom: 1.75rem;
  }
  h2 {
    font-size: 1rem;
    margin: 0 0 0.6rem;
  }

  .einfache-liste {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .einfache-liste li {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    flex-wrap: wrap;
    padding: 0.5rem 0.7rem;
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    background: var(--karten-hintergrund);
  }
  .referenz-typ {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-gedaempft);
  }
  .referenz-ziel {
    flex: 1;
    min-width: 10rem;
  }
  .alter {
    color: var(--text-gedaempft);
    font-size: 0.8rem;
    margin-left: auto;
    white-space: nowrap;
  }

  .badge {
    font-size: 0.75rem;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    border: 1px solid var(--rahmen);
    white-space: nowrap;
  }
  .badge--ok {
    color: var(--farbe-ruhig);
    border-color: var(--farbe-ruhig);
  }
  .badge--nicht_erreichbar {
    color: var(--farbe-ueberfaellig);
    border-color: var(--farbe-ueberfaellig);
  }
  .badge--nicht_pruefbar {
    color: var(--text-gedaempft);
  }
  .badge--stufe-ueberall {
    color: var(--akzent);
    border-color: var(--akzent);
  }
  .badge--stufe-nur_desktop {
    color: var(--farbe-auffaellig);
    border-color: var(--farbe-auffaellig);
  }
  .badge--quelle {
    color: var(--akzent);
    border-color: var(--akzent);
  }

  .logbuch {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .logbuch li {
    padding: 0.7rem 0.9rem;
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    background: var(--karten-hintergrund);
  }
  .logbuch-kopf {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.35rem;
  }
  .logbuch-kopf .alter {
    margin-left: auto;
  }
</style>
