<script lang="ts">
  // Ein Vorhaben anlegen. Ein Feld, eine Frage: **Was gibt's?**
  //
  // Das Feld nimmt alles – einen Ordner, einen Link, eine Datei, oder einfach einen Namen.
  // Die Einordnung macht der Kern (`deuten::einordnen`), nicht diese Datei: dort ist sie
  // geprüft, und sie muss auf dem Dateisystem nachsehen. Wer fragt »ist das ein Ordner
  // oder ein Titel?«, verlangt eine Einordnung, die der Mensch gerade nicht vorhatte.
  //
  // Danach ein Befund mit Häkchen: was gefunden wurde, nie eine zweite Fragerunde.
  //
  // Zur Vorsicht, weil es genau hier schon einmal schiefging: der Rücksetz-Effekt darf
  // **keine** `bind:this`-Referenz lesen. Svelte schreibt die, nachdem der Dialog steht,
  // und beim Wechsel auf den Befund wieder auf `undefined` – ein Effekt, der sie liest,
  // läuft dann noch einmal und macht seine eigene Rücksetzung zunichte. In 0.8.0 sprang
  // der Dialog dadurch im selben Frame zurück, und beide Knöpfe sahen aus wie tot.
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
    type FundStartseite,
    type FundUnterprojekt,
  } from '../data/tauri';
  import { meldungen } from '../meldung.svelte';
  import { VORLAGEN_LABEL } from '../format';
  import { navigiereZu } from '../router.svelte';
  import type { VorlagenId } from '../data/types';

  let { offen = $bindable(false) }: { offen?: boolean } = $props();

  const VORLAGEN = Object.keys(VORLAGEN_LABEL) as VorlagenId[];

  let eingabe = $state('');
  let deutung: Deutung | null = $state(null);
  /** Der gedeutete Ordner, falls die Quelle einer war – er bekommt die Markerdatei. */
  let ordnerPfad: string | undefined = $state();
  let titel = $state('');
  let vorlage: VorlagenId = $state('generisch');
  let kurs = $state('');
  let tags: string[] = $state([]);
  /** Häkchen je Fund, alle vorangekreuzt. */
  let gewaehlt: Record<string, boolean> = $state({});
  let laeuft = $state(false);
  let wirdGespeichert = $state(false);
  let fehler: string | null = $state(null);
  let ueberZiehen = $state(false);
  let eingabeEl: HTMLInputElement | undefined = $state();

  function schluessel(f: Fund): string {
    switch (f.art) {
      case 'remote':
        return `remote:${f.url}`;
      case 'unterprojekt':
        return `unterprojekt:${f.pfad}`;
      case 'dokument':
        return `dokument:${f.pfad}`;
      case 'startseite':
        return `startseite:${f.url}`;
      case 'schon_bekannt':
        return `bekannt:${f.pfad}`;
    }
  }

  // Zurücksetzen genau beim Öffnen. Liest nur `offen` – siehe Kopf dieser Datei.
  $effect(() => {
    if (!offen) return;
    eingabe = '';
    deutung = null;
    ordnerPfad = undefined;
    titel = '';
    vorlage = 'generisch';
    kurs = '';
    tags = [];
    gewaehlt = {};
    fehler = null;
    laeuft = false;
    ueberZiehen = false;
  });

  // Fokus als eigener Effekt: er liest die Referenz, schreibt aber nichts.
  $effect(() => {
    if (offen && !deutung) eingabeEl?.focus();
  });

  // Hineingezogene Ordner und Dateien. Tauri fängt das auf Fensterebene ab und liefert
  // echte Pfade – im Webview wären es Datei-Objekte ohne Pfad, mit denen der Kern nichts
  // anfangen kann.
  $effect(() => {
    if (!offen || !echteDaten) return;
    let abmelden: (() => void) | undefined;
    let entsorgt = false;
    void import('@tauri-apps/api/webview').then(({ getCurrentWebview }) =>
      getCurrentWebview()
        .onDragDropEvent((e) => {
          if (e.payload.type === 'enter' || e.payload.type === 'over') {
            ueberZiehen = true;
          } else if (e.payload.type === 'leave') {
            ueberZiehen = false;
          } else if (e.payload.type === 'drop') {
            ueberZiehen = false;
            // Nur im ersten Schritt: ein Wurf auf den Befund würde die Häkchen
            // wegwerfen, die gerade gesetzt wurden.
            if (!deutung) void deutePfade(e.payload.paths);
          }
        })
        .then((un) => {
          if (entsorgt) un();
          else abmelden = un;
        }),
    );
    return () => {
      entsorgt = true;
      abmelden?.();
    };
  });

  function uebernehmen(d: Deutung) {
    deutung = d;
    gewaehlt = Object.fromEntries(d.befund.funde.map((f) => [schluessel(f), true]));
    const v = d.befund.vorschlag;
    titel = v?.titel ?? '';
    vorlage = v?.vorlage ?? 'generisch';
    kurs = v?.kurs ?? '';
    tags = v?.tags ?? [];
    fehler = null;
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

  /**
   * Im Browser gibt es keinen Kern, der einordnen könnte – dort ist alles ein Titel.
   * Spiegelt `deuten::nur_titel` im Kern; die Beispieldaten sollen trotzdem etwas zeigen.
   */
  function nurTitel(text: string): Deutung {
    return {
      befund: {
        quelle: text.trim(),
        vorschlag: { titel: text.trim(), vorlage: 'generisch', tags: [] },
        funde: [],
        angesehen: 0,
        abgebrochen: false,
        weitere_dokumente: 0,
        archiviert: false,
      },
    };
  }

  async function weiter(e?: Event) {
    e?.preventDefault();
    if (laeuft) return;
    const text = eingabe.trim();
    if (!text) {
      fehler = 'Schreib etwas hinein: einen Ordner, einen Link oder einen Namen.';
      return;
    }
    ordnerPfad = undefined;
    if (!echteDaten) {
      uebernehmen(nurTitel(text));
      return;
    }
    await deute(async () => {
      const d = await deuten.eingabe({ art: 'text', text });
      ordnerPfad = ordnerAus(d);
      return d;
    });
  }

  async function deutePfade(pfade: string[]) {
    if (!pfade.length) return;
    eingabe = pfade.length === 1 ? pfade[0] : `${pfade.length} Dateien`;
    await deute(async () => {
      const d = await deuten.eingabe({ art: 'pfade', pfade });
      ordnerPfad = ordnerAus(d);
      return d;
    });
  }

  /**
   * Der Ordner, auf den der Befund sich bezieht – vom Kern beantwortet (`Befund::ordner`).
   *
   * Hier stand bis 0.10 eine eigene Herleitung, die Unterordner oder Unterprojekte
   * verlangte. Ein flacher Ordner mit ein paar PDFs – der Normalfall für alles außer
   * Software – galt damit als „kein Ordner“, und daran hingen die Ordner-Referenz, der
   * Import der Git-Historie und die Markerdatei. Zweimal dieselbe Frage zu beantworten
   * war der Fehler, nicht die Antwort.
   */
  function ordnerAus(d: Deutung): string | undefined {
    return d.ordner;
  }

  async function ordnerWaehlen() {
    const pfad = await system.ordnerWaehlen();
    if (!pfad) return;
    eingabe = pfad;
    ordnerPfad = pfad;
    await deute(() => deuten.quelle({ art: 'ordner', pfad }));
  }

  async function dateienWaehlen() {
    const pfade = await system.dateienWaehlen();
    if (!pfade.length) return;
    eingabe = pfade.length === 1 ? pfade[0] : `${pfade.length} Dateien`;
    ordnerPfad = undefined;
    await deute(() => deuten.quelle({ art: 'dateien', pfade }));
  }

  const funde = $derived.by((): Fund[] => deutung?.befund.funde ?? []);
  const remote = $derived.by(() => funde.find((f): f is FundRemote => f.art === 'remote'));
  const unterprojekte = $derived.by(() => funde.filter((f): f is FundUnterprojekt => f.art === 'unterprojekt'));
  const dokumente = $derived.by(() => funde.filter((f): f is FundDokument => f.art === 'dokument'));
  const startseite = $derived.by(() => funde.find((f): f is FundStartseite => f.art === 'startseite'));
  /** Gehört die Quelle schon zu einem Vorhaben? Dann wird angehängt, nicht angelegt. */
  const bekannt = $derived.by(() => deutung?.bekannt);
  const kannSpeichern = $derived(!!bekannt || titel.trim().length > 0);

  function gewaehltePfade(liste: { pfad: string }[]): string[] {
    return liste.filter((f) => gewaehlt[schluessel(f as Fund)]).map((f) => f.pfad);
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
      const b = await deuten.anlegen({
        titel: titel.trim(),
        kurs: kurs.trim() || undefined,
        vorlage,
        ordner: ordnerPfad,
        remote: remote && gewaehlt[schluessel(remote)] ? remote.url : undefined,
        startseite: startseite && gewaehlt[schluessel(startseite)] ? startseite.url : undefined,
        tags,
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
    deutung = null;
    ordnerPfad = undefined;
    fehler = null;
  }

  function aufTaste(e: KeyboardEvent) {
    if (e.key === 'Escape' && offen) offen = false;
  }

  function randText(b: Befund): string {
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

    {#if !deutung}
      <form class="dialog" class:ueber-ziehen={ueberZiehen} onsubmit={weiter} aria-label="Neues Vorhaben">
        <h2>Was gibt's?</h2>

        <input
          bind:this={eingabeEl}
          bind:value={eingabe}
          type="text"
          class="gross"
          placeholder="Ordner, Link, Datei – oder einfach ein Name"
          aria-label="Ordner, Link, Datei oder Name"
          autocomplete="off"
          spellcheck="false"
        />

        <p class="hinweis">
          Lotse sieht sich an, was du hineinschreibst, und füllt Titel, Kurs und Vorlage selbst aus.
          {#if echteDaten}
            Ordner und Dateien kannst du auch einfach hierher ziehen.
          {/if}
        </p>

        {#if fehler}
          <p class="fehler">{fehler}</p>
        {/if}

        <div class="aktionen">
          {#if echteDaten}
            <span class="durchsuchen">
              <span class="hinweis klein">Durchsuchen:</span>
              <button type="button" onclick={ordnerWaehlen} disabled={laeuft}>Ordner …</button>
              <button type="button" onclick={dateienWaehlen} disabled={laeuft}>Dateien …</button>
            </span>
          {/if}
          <button type="button" onclick={() => (offen = false)}>Abbrechen</button>
          <button type="submit" class="primaer" disabled={laeuft}>
            {laeuft ? 'Sehe nach …' : 'Weiter'}
          </button>
        </div>
      </form>
    {:else}
      <form class="dialog" onsubmit={anlegen} aria-label="Befund">
        {#if bekannt}
          <h2>Gehört schon zu „{bekannt.titel}"</h2>
          <p class="hinweis">
            Dort liegt eine Kennung von Lotse. Es entsteht also kein zweites Vorhaben – was unten angehakt bleibt,
            kommt zu „{bekannt.titel}" dazu.
          </p>
        {:else}
          <h2>Befund</h2>
          {#if deutung.befund.quelle && deutung.befund.quelle !== titel}
            <p class="hinweis"><code>{deutung.befund.quelle}</code></p>
          {/if}

          <label>
            <span>Titel</span>
            <input bind:value={titel} type="text" placeholder="Gartenhaus" required />
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

          {#if tags.length}
            <p class="tags">
              <span class="hinweis klein">Themen von der Gegenseite:</span>
              {#each tags as t (t)}
                <button
                  type="button"
                  class="tag"
                  title="Entfernen"
                  onclick={() => (tags = tags.filter((x) => x !== t))}
                >
                  {t} ×
                </button>
              {/each}
            </p>
          {/if}
        {/if}

        {#if deutung.befund.archiviert}
          <p class="hinweis warnung">
            Das Repo ist <strong>archiviert</strong> – dort passiert nichts mehr. Anlegen kannst du es trotzdem;
            Lotse wird nur nie Bewegung melden.
          </p>
        {/if}
        {#if deutung.befund.gegenseite_fehler}
          <p class="hinweis warnung">
            Von der Gegenseite kam nichts: {deutung.befund.gegenseite_fehler} Titel und Link stehen trotzdem; was
            dort steht, holt Lotse später nach.
          </p>
        {/if}

        {#if remote}
          <fieldset>
            <legend>Gegenseite</legend>
            <label class="haken">
              <input type="checkbox" bind:checked={gewaehlt[schluessel(remote)]} />
              <span>{remote.dienst ?? 'Adresse'} – <code>{remote.url}</code></span>
            </label>
            <p class="hinweis klein">
              Wird als Referenz angehängt. Damit Lotse dort auch nachsieht, braucht es später einen Zugang – das sagt
              es dann selbst, nicht jetzt.
            </p>
          </fieldset>
        {/if}

        {#if startseite}
          <fieldset>
            <legend>Projektseite</legend>
            <label class="haken">
              <input type="checkbox" bind:checked={gewaehlt[schluessel(startseite)]} />
              <span><code>{startseite.url}</code></span>
            </label>
          </fieldset>
        {/if}

        {#if unterprojekte.length}
          <fieldset>
            <legend>Eigene Vorhaben darin ({unterprojekte.length})</legend>
            <p class="hinweis klein">Jedes abgehakte wird ein eigenes Vorhaben. Was weg soll, hier abwählen.</p>
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

        {#if !bekannt && funde.length === 0}
          <p class="hinweis klein">
            Nichts weiter gefunden – das ist kein Mangel. Ordner, Adressen und Dateien lassen sich später unter
            <em>Referenzen</em> nachtragen.
          </p>
        {/if}

        {#if deutung.befund.abgebrochen}
          <p class="hinweis klein">
            Die Suche hat an ihrer Grenze aufgehört – der Ordner ist groß. Was tiefer liegt, steht nicht in dieser
            Liste; einzelne Unterordner lassen sich später nachtragen.
          </p>
        {/if}
        {#if randText(deutung.befund)}
          <p class="hinweis klein">{randText(deutung.befund)}</p>
        {/if}

        {#if fehler}
          <p class="fehler">{fehler}</p>
        {/if}

        <div class="aktionen">
          <button type="button" onclick={zurueck}>Zurück</button>
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
  /* Sichtbar machen, dass der Wurf hier ankommt. */
  .dialog.ueber-ziehen {
    border-color: var(--akzent, currentColor);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3), 0 0 0 2px var(--akzent, currentColor);
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
  /* Das eine Feld, das alles annimmt: es darf aussehen wie die Hauptsache. */
  input.gross {
    font-size: 1.05rem;
    padding: 0.6rem 0.7rem;
  }
  select {
    color: inherit;
    background: var(--hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.4rem;
    padding: 0.5rem 0.6rem;
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
  code {
    font-size: 0.78rem;
    word-break: break-all;
  }
  .aktionen {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .durchsuchen {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-right: auto;
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
  .hinweis.warnung {
    color: var(--farbe-ueberfaellig);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    align-items: center;
    margin: 0;
  }
  .tag {
    font: inherit;
    font-size: 0.78rem;
    border: 1px solid var(--rahmen);
    background: transparent;
    color: inherit;
    border-radius: 999px;
    padding: 0.1rem 0.55rem;
    cursor: pointer;
  }
</style>
