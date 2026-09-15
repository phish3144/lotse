<script lang="ts">
  // Ein Projekt anlegen: erst die Herkunft, dann der Befund.
  //
  // Der alte Dialog fragte nach Titel, Vorlage und Kurs – also nach genau dem, was Lotse
  // aus einem Ordner selbst herauslesen kann. Jetzt kommt zuerst die einzige Frage, die
  // niemand sonst beantworten kann: woher kommt das? Alles Weitere ist ein Befund mit
  // Häkchen, keine zweite Fragerunde. Was Lotse schon weiß, wird nicht erfragt: hat der
  // Ordner ein Remote, steht es da, statt dass jemand »von GitHub importieren?« bejaht.
  import { echteDaten, provider } from '../data/store';
  import { datenVersion } from '../data/version.svelte';
  import {
    deuten,
    system,
    type Befund,
    type Deutung,
    type Fund,
    type FundDokument,
    type FundRemote,
    type FundUnterprojekt,
  } from '../data/tauri';
  import { meldungen } from '../meldung.svelte';
  import { VORLAGEN_LABEL } from '../format';
  import { navigiereZu } from '../router.svelte';
  import type { VorlagenId } from '../data/types';

  let { offen = $bindable(false) }: { offen?: boolean } = $props();

  const VORLAGEN = Object.keys(VORLAGEN_LABEL) as VorlagenId[];

  type Schritt = 'herkunft' | 'befund';

  let schritt: Schritt = $state('herkunft');
  let deutung: Deutung | null = $state(null);
  /** Der gedeutete Ordner. Nur gesetzt, wenn die Quelle einer war. */
  let ordnerPfad: string | undefined = $state();
  let adresse = $state('');
  let titel = $state('');
  let vorlage: VorlagenId = $state('generisch');
  let kurs = $state('');
  /** Häkchen je Fund, alle vorangekreuzt. Schlüssel ist `schluessel(fund)`. */
  let gewaehlt: Record<string, boolean> = $state({});
  let laeuft = $state(false);
  let wirdGespeichert = $state(false);
  let fehler: string | null = $state(null);
  let titelEl: HTMLInputElement | undefined = $state();
  let adresseEl: HTMLInputElement | undefined = $state();

  function schluessel(f: Fund): string {
    switch (f.art) {
      case 'remote':
        return `remote:${f.url}`;
      case 'unterprojekt':
        return `unterprojekt:${f.pfad}`;
      case 'dokument':
        return `dokument:${f.pfad}`;
      case 'schon_bekannt':
        return `bekannt:${f.pfad}`;
    }
  }

  $effect(() => {
    if (!offen) return;
    // Ohne Hülle gibt es keine Ordner und keine Dialoge – dann bleibt das Formular,
    // und der Schritt „Herkunft" wäre eine leere Seite.
    schritt = echteDaten ? 'herkunft' : 'befund';
    deutung = echteDaten ? null : ohneQuelle();
    ordnerPfad = undefined;
    adresse = '';
    titel = '';
    vorlage = 'generisch';
    kurs = '';
    gewaehlt = {};
    fehler = null;
    laeuft = false;
    (echteDaten ? adresseEl : titelEl)?.focus();
  });

  /** Ein Befund ohne Quelle: das alte Formular, als Sonderfall des neuen Ablaufs. */
  function ohneQuelle(): Deutung {
    return {
      befund: {
        quelle: 'ohne Quelle',
        vorschlag: { titel: '', vorlage: 'generisch' },
        funde: [],
        angesehen: 0,
        abgebrochen: false,
        weitere_dokumente: 0,
      },
    };
  }

  function uebernehmen(d: Deutung) {
    deutung = d;
    gewaehlt = Object.fromEntries(d.befund.funde.map((f) => [schluessel(f), true]));
    const v = d.befund.vorschlag;
    titel = v?.titel ?? '';
    vorlage = v?.vorlage ?? 'generisch';
    kurs = v?.kurs ?? '';
    schritt = 'befund';
    fehler = null;
  }

  async function deuteOrdner() {
    const pfad = await system.ordnerWaehlen();
    if (!pfad) return;
    ordnerPfad = pfad;
    await deute(() => deuten.quelle({ art: 'ordner', pfad }));
  }

  async function deuteAdresse() {
    const url = adresse.trim();
    if (!url) return;
    ordnerPfad = undefined;
    await deute(() => deuten.quelle({ art: 'adresse', url }));
  }

  async function deuteDateien() {
    const pfade = await system.dateienWaehlen();
    if (!pfade.length) return;
    ordnerPfad = undefined;
    await deute(() => deuten.quelle({ art: 'dateien', pfade }));
  }

  async function deute(auftrag: () => Promise<Deutung>) {
    laeuft = true;
    fehler = null;
    try {
      uebernehmen(await auftrag());
    } catch (e) {
      fehler = e instanceof Error ? e.message : String(e);
    } finally {
      laeuft = false;
    }
  }

  // `$derived.by` statt `$derived`: im Ausdruck hält TypeScript `deutung` noch für das
  // `null` der Deklaration, im Funktionsrumpf nicht mehr.
  const funde = $derived.by((): Fund[] => deutung?.befund.funde ?? []);
  const remote = $derived.by(() => funde.find((f): f is FundRemote => f.art === 'remote'));
  const unterprojekte = $derived.by(() => funde.filter((f): f is FundUnterprojekt => f.art === 'unterprojekt'));
  const dokumente = $derived.by(() => funde.filter((f): f is FundDokument => f.art === 'dokument'));
  /** Gehört die Quelle schon zu einem Projekt? Dann wird angehängt, nicht angelegt. */
  const bekannt = $derived.by(() => deutung?.bekannt);
  const kannSpeichern = $derived(!!bekannt || titel.trim().length > 0);

  function gewaehltePfade(liste: Fund[]): string[] {
    return liste
      .filter((f) => gewaehlt[schluessel(f)])
      .map((f) => ('pfad' in f ? f.pfad : ''))
      .filter((p) => p.length > 0);
  }

  async function anlegen(e: Event) {
    e.preventDefault();
    if (!kannSpeichern || wirdGespeichert) return;
    wirdGespeichert = true;
    fehler = null;
    try {
      if (!echteDaten) {
        const p = await provider.createProject(titel.trim(), vorlage, kurs.trim() || undefined);
        fertig(p.id, `„${p.titel}" angelegt.`);
        return;
      }
      const remoteGewaehlt = remote && gewaehlt[schluessel(remote)] ? remote.url : undefined;
      const b = await deuten.anlegen({
        titel: titel.trim(),
        kurs: kurs.trim() || undefined,
        vorlage,
        ordner: ordnerPfad,
        remote: remoteGewaehlt,
        unterprojekte: gewaehltePfade(unterprojekte),
        dokumente: gewaehltePfade(dokumente),
        an_projekt: bekannt?.id,
      });
      fertig(b.projekt.id, bilanzText(b.projekt.titel, b.unterprojekte.length, b.referenzen, !!bekannt));
    } catch (e2) {
      fehler = e2 instanceof Error ? e2.message : String(e2);
    } finally {
      wirdGespeichert = false;
    }
  }

  function bilanzText(name: string, unter: number, referenzen: number, angehaengt: boolean): string {
    const teile: string[] = [];
    if (referenzen === 1) teile.push('1 Referenz');
    else if (referenzen > 1) teile.push(`${referenzen} Referenzen`);
    if (unter === 1) teile.push('1 Unterprojekt');
    else if (unter > 1) teile.push(`${unter} Unterprojekte`);
    const kopf = angehaengt ? `An „${name}" angehängt` : `„${name}" angelegt`;
    return teile.length ? `${kopf}: ${teile.join(', ')}.` : `${kopf}.`;
  }

  function fertig(id: string, text: string) {
    datenVersion.bump();
    offen = false;
    meldungen.zeigen(text);
    navigiereZu(`#/projekt/${id}`);
  }

  function zurueck() {
    schritt = 'herkunft';
    deutung = null;
    ordnerPfad = undefined;
    fehler = null;
  }

  function aufTaste(e: KeyboardEvent) {
    if (e.key === 'Escape' && offen) offen = false;
  }

  function markenText(b: Befund): string {
    const teile: string[] = [];
    if (b.angesehen > 0) teile.push(`${b.angesehen} Ordner angesehen`);
    if (b.weitere_dokumente > 0) teile.push(`${b.weitere_dokumente} weitere Dokumente nicht aufgeführt`);
    return teile.join(' · ');
  }
</script>

<svelte:window onkeydown={aufTaste} />

{#if offen}
  <div class="ueberlagerung">
    <button type="button" class="rueckwand" aria-label="Dialog schließen" onclick={() => (offen = false)}></button>

    {#if schritt === 'herkunft'}
      <div class="dialog" role="dialog" aria-label="Neues Projekt: Herkunft">
        <h2>Woher kommt es?</h2>
        <p class="hinweis">
          Lotse sieht sich die Quelle an und schlägt Titel, Kurs und Vorlage selbst vor. Gefragt wird danach nur noch,
          was es behalten soll.
        </p>

        <div class="wege">
          <button type="button" class="weg" onclick={deuteOrdner} disabled={laeuft}>
            <strong>Ordner</strong>
            <span>Ein Ordner auf diesem Gerät. Unterprojekte, Git-Historie und Dokumente kommen mit.</span>
          </button>
          <button type="button" class="weg" onclick={deuteDateien} disabled={laeuft}>
            <strong>Dateien</strong>
            <span>PDFs, Notizen, Tabellen. Werden als Referenzen angehängt.</span>
          </button>
        </div>

        <label>
          <span>Adresse eines Repos oder einer Seite</span>
          <span class="zeile">
            <input
              bind:this={adresseEl}
              bind:value={adresse}
              type="text"
              placeholder="https://github.com/name/vorhaben"
              onkeydown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  deuteAdresse();
                }
              }}
            />
            <button type="button" onclick={deuteAdresse} disabled={laeuft || !adresse.trim()}>Deuten</button>
          </span>
        </label>

        {#if laeuft}
          <p class="hinweis" role="status" aria-live="polite">Sehe nach …</p>
        {/if}
        {#if fehler}
          <p class="fehler">{fehler}</p>
        {/if}

        <div class="aktionen">
          <button type="button" onclick={() => (offen = false)}>Abbrechen</button>
          <button type="button" onclick={() => uebernehmen(ohneQuelle())}>Ohne Quelle</button>
        </div>
      </div>
    {:else if deutung}
      <form class="dialog" onsubmit={anlegen} aria-label="Neues Projekt: Befund">
        {#if bekannt}
          <h2>Gehört schon zu „{bekannt.titel}"</h2>
          <p class="hinweis">
            In diesem Ordner liegt eine Kennung von Lotse. Es entsteht also kein zweites Projekt – was unten
                        angehakt bleibt, kommt zu „{bekannt.titel}" dazu.
          </p>
        {:else}
          <h2>Befund</h2>
          <p class="hinweis">{deutung.befund.quelle}</p>

          <label>
            <span>Titel</span>
            <input bind:this={titelEl} bind:value={titel} type="text" placeholder="Gartenhaus" required />
          </label>

          <label>
            <span>Vorlage</span>
            <select bind:value={vorlage}>
              {#each VORLAGEN as v (v)}
                <option value={v}>{VORLAGEN_LABEL[v]}</option>
              {/each}
            </select>
          </label>

          <label>
            <span>Kurs <em>(optional)</em></span>
            <textarea bind:value={kurs} rows="2" placeholder="Worum geht es? Was ist das Ziel?"></textarea>
          </label>
        {/if}

        {#if ordnerPfad}
          <p class="fund-zeile"><span class="marke">Ordner</span> <code>{ordnerPfad}</code></p>
        {/if}

        {#if remote}
          <fieldset>
            <legend>Gegenseite</legend>
            <label class="haken">
              <input type="checkbox" bind:checked={gewaehlt[schluessel(remote)]} />
              <span>
                {remote.dienst ?? 'Adresse'} – <code>{remote.url}</code>
              </span>
            </label>
            <p class="hinweis klein">
              Wird als Referenz angehängt. Damit Lotse dort auch nachsieht, braucht es später einen Zugang – das sagt
              es dann selbst, nicht jetzt.
            </p>
          </fieldset>
        {/if}

        {#if unterprojekte.length}
          <fieldset>
            <legend>Eigene Vorhaben darin ({unterprojekte.length})</legend>
            <p class="hinweis klein">Jedes abgehakte wird ein eigenes Projekt. Was weg soll, hier abwählen.</p>
            <div class="liste">
              {#each unterprojekte as f (f.pfad)}
                <label class="haken">
                  <input type="checkbox" bind:checked={gewaehlt[schluessel(f)]} />
                  <span>
                    <strong>{f.name}</strong>
                    <em>{VORLAGEN_LABEL[f.vorlage]}</em>
                    {#if f.marken.length}<em class="marken">{f.marken.join(', ')}</em>{/if}
                  </span>
                </label>
              {/each}
            </div>
          </fieldset>
        {/if}

        {#if dokumente.length}
          <fieldset>
            <legend>Dokumente ({dokumente.length})</legend>
            <div class="liste">
              {#each dokumente as f (f.pfad)}
                <label class="haken">
                  <input type="checkbox" bind:checked={gewaehlt[schluessel(f)]} />
                  <span>{f.name}</span>
                </label>
              {/each}
            </div>
          </fieldset>
        {/if}

        {#if deutung.befund.abgebrochen}
          <p class="hinweis klein">
            Die Suche hat an ihrer Grenze aufgehört – der Ordner ist groß. Was tiefer liegt, steht nicht in dieser
            Liste; einzelne Unterordner lassen sich später nachtragen.
          </p>
        {/if}
        {#if markenText(deutung.befund)}
          <p class="hinweis klein">{markenText(deutung.befund)}</p>
        {/if}

        {#if fehler}
          <p class="fehler">{fehler}</p>
        {/if}

        <div class="aktionen">
          {#if echteDaten}
            <button type="button" onclick={zurueck}>Zurück</button>
          {:else}
            <button type="button" onclick={() => (offen = false)}>Abbrechen</button>
          {/if}
          <button type="submit" class="primaer" disabled={!kannSpeichern || wirdGespeichert}>
            {#if wirdGespeichert}
              {bekannt ? 'Hänge an …' : 'Lege an …'}
            {:else}
              {bekannt ? 'Anhängen' : 'Anlegen'}
            {/if}
          </button>
        </div>
      </form>
    {/if}
  </div>
{/if}

<style>
  .ueberlagerung {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 8vh;
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
    padding: 1.2rem 1.3rem 1.3rem;
    width: min(34rem, 92vw);
    max-height: 80vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }
  h2 {
    margin: 0;
    font-size: 1.15rem;
  }
  .hinweis {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text-gedaempft);
  }
  .hinweis.klein {
    font-size: 0.78rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.9rem;
  }
  label > span:first-child {
    color: var(--text-gedaempft);
  }
  label em {
    font-style: normal;
    opacity: 0.7;
  }
  input[type='text'],
  textarea,
  select {
    font: inherit;
    width: 100%;
  }
  select {
    color: inherit;
    background: var(--hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.4rem;
    padding: 0.5rem 0.6rem;
  }
  .zeile {
    display: flex;
    gap: 0.4rem;
    align-items: stretch;
  }
  .zeile button {
    border: 1px solid var(--rahmen);
    background: transparent;
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.4rem 0.8rem;
    white-space: nowrap;
  }
  /* Die Wege sind gleichwertig und gleich groß: keiner ist der empfohlene. */
  .wege {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: 0.5rem;
  }
  .weg {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    text-align: left;
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    background: transparent;
    color: inherit;
    padding: 0.7rem 0.8rem;
    cursor: pointer;
  }
  .weg:hover:not(:disabled) {
    border-color: var(--text-gedaempft);
  }
  .weg:disabled {
    opacity: 0.5;
  }
  .weg span {
    font-size: 0.78rem;
    color: var(--text-gedaempft);
  }
  fieldset {
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    padding: 0.6rem 0.8rem 0.7rem;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  legend {
    font-size: 0.8rem;
    color: var(--text-gedaempft);
    padding: 0 0.3rem;
  }
  .liste {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    max-height: 13rem;
    overflow-y: auto;
  }
  .haken {
    flex-direction: row;
    align-items: baseline;
    gap: 0.45rem;
  }
  .haken input {
    margin: 0;
    flex: none;
  }
  .haken span {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: baseline;
    font-size: 0.85rem;
  }
  .haken em.marken {
    font-size: 0.75rem;
  }
  .fund-zeile {
    margin: 0;
    font-size: 0.8rem;
    display: flex;
    gap: 0.4rem;
    align-items: baseline;
    flex-wrap: wrap;
  }
  .marke {
    color: var(--text-gedaempft);
  }
  code {
    font-size: 0.78rem;
    word-break: break-all;
  }
  .aktionen {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .aktionen button {
    border: 1px solid var(--rahmen);
    background: transparent;
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.45rem 0.9rem;
  }
  .aktionen button:disabled {
    opacity: 0.5;
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
    font-size: 0.85rem;
    margin: 0;
  }
</style>
