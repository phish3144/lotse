<script lang="ts">
  import { computeBrief, sollBriefZeigen } from '../lib/brief';
  import NeuerTresorEintrag from '../lib/components/NeuerTresorEintrag.svelte';
  import NoteText from '../lib/components/NoteText.svelte';
  import TresorListe from '../lib/components/TresorListe.svelte';
  import { echteDaten, provider } from '../lib/data/store';
  import { datei, forge, kalender, system, type DateiAuszug, type ForgeProjekt, type Termin } from '../lib/data/tauri';
  import { datenVersion } from '../lib/data/version.svelte';
  import { meldungen } from '../lib/meldung.svelte';
  import { navigiereZu } from '../lib/router.svelte';
  import {
    alterInTagenText,
    ANLEGBARE_REFERENZ_TYPEN,
    ART_LABEL,
    datumText,
    PRUEFSTATUS_LABEL,
    QUELLE_LABEL,
    REFERENZ_ROLLE_LABEL,
    REFERENZ_TYP_LABEL,
    STATUS_LABEL,
    tageText,
    VORLAGEN_LABEL,
  } from '../lib/format';
  import type { Id, Notiz, NotizArt, Projekt, ProjektStatus, ReferenzRolle, ReferenzTyp } from '../lib/data/types';

  let { id }: { id: string } = $props();

  const STATUS_OPTIONEN: ProjektStatus[] = ['idee', 'aktiv', 'pausiert', 'wartet', 'abgeschlossen', 'eingemottet'];
  const ERFASSBARE_ARTEN: NotizArt[] = ['log', 'offen', 'entscheidung'];
  const REFERENZ_TYPEN = ANLEGBARE_REFERENZ_TYPEN;
  const REFERENZ_ROLLEN = Object.keys(REFERENZ_ROLLE_LABEL) as ReferenzRolle[];

  /** Referenzen, die das System öffnen kann. Ein Regal im Keller kann es nicht. */
  const OEFFENBAR: ReferenzTyp[] = ['ordner', 'git_repo', 'url', 'datei'];

  async function laden(projektId: string) {
    const [projekt, notizen, referenzen, zugaenge, projekte] = await Promise.all([
      provider.getProject(projektId),
      provider.listNotes(projektId),
      provider.listReferences(projektId),
      provider.listVaultEntries(projektId),
      provider.listProjects(),
    ]);
    return { projekt, notizen, referenzen, zugaenge, projekte };
  }

  let datenPromise = $derived.by(() => {
    datenVersion.wert;
    return laden(id);
  });

  let fehlerText: string | null = $state(null);
  let wirdGespeichert = $state(false);

  function melde(e: unknown) {
    fehlerText = e instanceof Error ? e.message : String(e);
    meldungen.fehler(e);
  }

  // --- Referenz öffnen, Notiz umsortieren, Projekt löschen ------------------
  let verschiebeNotiz: Id | null = $state(null);
  let loeschenGefragt = $state(false);

  async function referenzOeffnen(ziel: string) {
    try {
      await system.oeffnen(ziel);
    } catch (e) {
      melde(e);
    }
  }

  async function notizVerschieben(notizId: Id, zielId: Id) {
    verschiebeNotiz = null;
    try {
      await provider.moveNote(notizId, zielId);
      datenVersion.bump();
      meldungen.zeigen('Notiz verschoben.');
    } catch (e) {
      melde(e);
    }
  }

  async function projektLoeschen(titel: string) {
    wirdGespeichert = true;
    try {
      await provider.deleteProject(id);
      datenVersion.bump();
      meldungen.zeigen(`„${titel}“ gelöscht.`);
      navigiereZu('#/');
    } catch (e) {
      melde(e);
    } finally {
      wirdGespeichert = false;
      loeschenGefragt = false;
    }
  }

  // --- Statuswechsel: braucht bei pausiert/wartet eine Übergabenotiz (CONCEPT.md 4) ---
  let menuOffen = $state(false);
  let ausstehenderStatus: ProjektStatus | null = $state(null);
  let uebergabeText = $state('');
  let wiedervorlage = $state('');

  function brauchtUebergabe(status: ProjektStatus): boolean {
    return status === 'pausiert' || status === 'wartet';
  }

  function statusWaehlen(status: ProjektStatus, offeneFaeden: Notiz[]) {
    menuOffen = false;
    fehlerText = null;
    if (brauchtUebergabe(status)) {
      ausstehenderStatus = status;
      uebergabeText = offeneFaeden.length > 0 ? offeneFaeden.map((f) => `- ${f.text}`).join('\n') : '';
      wiedervorlage = '';
    } else {
      void speichereStatus(status);
    }
  }

  async function speichereStatus(status: ProjektStatus, text?: string, datum?: string) {
    wirdGespeichert = true;
    fehlerText = null;
    try {
      await provider.setStatus(id, status, text, datum || undefined);
      ausstehenderStatus = null;
      uebergabeText = '';
      wiedervorlage = '';
      datenVersion.bump();
    } catch (e) {
      melde(e);
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
    void speichereStatus(ausstehenderStatus, uebergabeText, wiedervorlage);
  }

  // --- Notiz erfassen -------------------------------------------------------
  let notizText = $state('');
  let notizArt: NotizArt = $state('log');

  async function notizSpeichern() {
    const text = notizText.trim();
    if (!text || wirdGespeichert) return;
    wirdGespeichert = true;
    fehlerText = null;
    try {
      await provider.addNote(id, { quelle: 'mensch', art: notizArt, text });
      notizText = '';
      notizArt = 'log';
      datenVersion.bump();
      meldungen.zeigen('Eingetragen.');
    } catch (e) {
      melde(e);
    } finally {
      wirdGespeichert = false;
    }
  }

  function aufNotizTaste(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      void notizSpeichern();
    }
  }

  async function fadenErledigen(notizId: Id) {
    fehlerText = null;
    try {
      await provider.completeThread(notizId);
      datenVersion.bump();
      meldungen.zeigen('Faden abgehakt.');
    } catch (e) {
      melde(e);
    }
  }

  // --- Projektkopf bearbeiten ----------------------------------------------
  let bearbeiten = $state(false);
  let eTitel = $state('');
  let eKurs = $state('');
  let eIntervall = $state(30);
  let eWiedervorlage = $state('');
  let eTags = $state('');

  function bearbeitenStarten(p: Projekt) {
    eTitel = p.titel;
    eKurs = p.kurs;
    eIntervall = p.erwartungsintervall_tage;
    eWiedervorlage = p.wiedervorlage ?? '';
    eTags = p.tags.join(', ');
    bearbeiten = true;
    fehlerText = null;
  }

  async function kopfSpeichern(p: Projekt) {
    if (!eTitel.trim() || wirdGespeichert) return;
    wirdGespeichert = true;
    fehlerText = null;
    try {
      await provider.saveProject({
        ...p,
        titel: eTitel.trim(),
        kurs: eKurs.trim(),
        erwartungsintervall_tage: Math.max(1, Number(eIntervall) || p.erwartungsintervall_tage),
        wiedervorlage: eWiedervorlage.trim() || undefined,
        tags: eTags
          .split(',')
          .map((t) => t.trim())
          .filter(Boolean),
      });
      bearbeiten = false;
      datenVersion.bump();
      meldungen.zeigen('Gespeichert.');
    } catch (e) {
      melde(e);
    } finally {
      wirdGespeichert = false;
    }
  }

  // --- Referenzen -----------------------------------------------------------
  let referenzFormular = $state(false);
  let rTyp: ReferenzTyp = $state('ordner');
  let rZiel = $state('');
  let rRolle: ReferenzRolle = $state('material');
  let geprueftWird: Id | null = $state(null);

  async function referenzSpeichern(e: Event) {
    e.preventDefault();
    if (!rZiel.trim() || wirdGespeichert) return;
    wirdGespeichert = true;
    fehlerText = null;
    try {
      await provider.addReference(id, rTyp, rZiel.trim(), rRolle);
      rZiel = '';
      referenzFormular = false;
      datenVersion.bump();
      meldungen.zeigen('Referenz angelegt.');
    } catch (e2) {
      melde(e2);
    } finally {
      wirdGespeichert = false;
    }
  }

  async function referenzPruefen(refId: Id) {
    geprueftWird = refId;
    fehlerText = null;
    try {
      await provider.checkReference(refId);
      datenVersion.bump();
    } catch (e) {
      melde(e);
    } finally {
      geprueftWird = null;
    }
  }

  // --- Gegenseite (GitHub) --------------------------------------------------
  // Das Repo kommt aus einer eingetragenen Adresse oder aus dem Git-Remote eines
  // Ordners; niemand muss es von Hand hinterlegen.
  let gegenseite: ForgeProjekt | null = $state(null);
  let gegenseiteLaeuft = $state(false);

  $effect(() => {
    const projektId = id;
    gegenseite = null;
    if (!echteDaten) return;
    let gilt = true;
    forge
      .projekt(projektId)
      .then((p) => {
        if (gilt) gegenseite = p;
      })
      .catch(() => {});
    return () => {
      gilt = false;
    };
  });

  async function gegenseiteAbfragen() {
    gegenseiteLaeuft = true;
    try {
      const r = await forge.abfragen(id);
      for (const f of r.fehler) meldungen.zeigen(f, 'fehler');
      if (r.notizen > 0) {
        datenVersion.bump();
        meldungen.zeigen('Stand der Gegenseite im Logbuch.');
      } else if (r.abgefragt > 0) {
        meldungen.zeigen('Abgefragt, nichts Neues zu melden.');
      }
    } catch (e) {
      melde(e);
    } finally {
      gegenseiteLaeuft = false;
    }
  }

  // --- Termine --------------------------------------------------------------
  // Aus abonnierten Kalendern (.ics). Lotse zeigt sie nur; gepflegt werden sie dort,
  // wo sie herkommen.
  let termine: Termin[] = $state([]);
  let terminFehler: string[] = $state([]);
  let terminQuellen = $state(0);

  $effect(() => {
    const projektId = id;
    termine = [];
    terminFehler = [];
    terminQuellen = 0;
    if (!echteDaten) return;
    let gilt = true;
    kalender
      .termine(projektId)
      .then((r) => {
        if (!gilt) return;
        termine = r.termine;
        terminFehler = r.fehler;
        terminQuellen = r.quellen.length;
      })
      .catch(() => {});
    return () => {
      gilt = false;
    };
  });

  function terminZeit(t: Termin): string {
    if (!t.uhrzeit) return 'ganztägig';
    return t.utc ? `${t.uhrzeit} UTC` : t.uhrzeit;
  }

  // --- Datei deuten ---------------------------------------------------------
  // Derselbe Weg wie bei der Verdichtung: erst zeigen, was gesendet würde, dann senden,
  // und ins Logbuch kommt es nur, wenn ein Mensch es übernimmt. Der Unterschied ist
  // allein, woher der Text stammt – aus einer Datei, die jemand ausdrücklich hergibt.
  let auszug: DateiAuszug | null = $state(null);
  let auszugErgebnis: string | null = $state(null);
  let auszugLaeuft = $state(false);
  let dateiVorschlaege: string[] = $state([]);

  $effect(() => {
    const projektId = id;
    dateiVorschlaege = [];
    if (!echteDaten) return;
    let gilt = true;
    datei
      .referenzen(projektId)
      .then((p) => {
        if (gilt) dateiVorschlaege = p;
      })
      .catch(() => {});
    return () => {
      gilt = false;
    };
  });

  async function dateiOeffnen(pfad: string | null) {
    if (!pfad) return;
    auszugLaeuft = true;
    auszugErgebnis = null;
    try {
      auszug = await datei.auszug(pfad);
    } catch (e) {
      melde(e);
    } finally {
      auszugLaeuft = false;
    }
  }

  async function dateiWaehlen() {
    try {
      await dateiOeffnen(await system.dateiWaehlen('Datei zum Deuten'));
    } catch (e) {
      melde(e);
    }
  }

  async function auszugSenden() {
    if (!auszug) return;
    auszugLaeuft = true;
    try {
      auszugErgebnis = await provider.kiVerdichten(
        `Datei: ${auszug.name}\n\n${auszug.text}`,
        'datei_deuten',
      );
    } catch (e) {
      melde(e);
    } finally {
      auszugLaeuft = false;
    }
  }

  async function auszugUebernehmen() {
    if (!auszugErgebnis || !auszug) return;
    try {
      await provider.addNote(id, {
        quelle: 'ki',
        art: 'log',
        text: `${auszug.name}: ${auszugErgebnis}`,
      });
      auszug = null;
      auszugErgebnis = null;
      datenVersion.bump();
      meldungen.zeigen('Im Logbuch.');
    } catch (e) {
      melde(e);
    }
  }

  function auszugVerwerfen() {
    auszug = null;
    auszugErgebnis = null;
  }

  // --- KI-Verdichtung -------------------------------------------------------
  // Erst zeigen, was gesendet würde, dann senden. Das Ergebnis ist ein Vorschlag;
  // ins Logbuch kommt es nur, wenn der Mensch es übernimmt.
  let kiText: string | null = $state(null);
  let kiErgebnis: string | null = $state(null);
  let kiLaeuft = $state(false);

  async function kiVorbereiten() {
    kiLaeuft = true;
    kiErgebnis = null;
    try {
      kiText = await provider.kiAnfrageText(id);
    } catch (e) {
      melde(e);
    } finally {
      kiLaeuft = false;
    }
  }

  async function kiSenden() {
    if (!kiText) return;
    kiLaeuft = true;
    try {
      kiErgebnis = await provider.kiVerdichten(kiText, 'brief_verdichten');
    } catch (e) {
      melde(e);
    } finally {
      kiLaeuft = false;
    }
  }

  async function kiUebernehmen() {
    if (!kiErgebnis) return;
    try {
      await provider.addNote(id, { quelle: 'ki', art: 'log', text: kiErgebnis });
      kiText = null;
      kiErgebnis = null;
      datenVersion.bump();
      meldungen.zeigen('Ins Logbuch übernommen.');
    } catch (e) {
      melde(e);
    }
  }

  function kiVerwerfen() {
    kiText = null;
    kiErgebnis = null;
  }

  // --- Tresor ---------------------------------------------------------------
  let tresorOffen = $state(false);

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
{:then { projekt, notizen, referenzen, zugaenge, projekte }}
  {#if !projekt}
    <p class="hinweis fehler">Projekt nicht gefunden.</p>
  {:else}
    {@const offeneFaeden = notizen.filter((n) => n.art === 'offen' && !n.erledigt_am)}
    {@const brief = sollBriefZeigen(projekt, notizen) ? computeBrief(projekt, notizen) : null}

    <NeuerTresorEintrag bind:offen={tresorOffen} projekte={[projekt]} vorausgewaehlt={projekt.id} />

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

        {#if echteDaten}
          {#if !kiText}
            <button type="button" class="schlicht ki-knopf" onclick={kiVorbereiten} disabled={kiLaeuft}>
              {kiLaeuft ? 'Moment …' : 'Von der KI verdichten lassen'}
            </button>
          {:else}
            <div class="ki">
              <strong>Das würde gesendet:</strong>
              <pre class="ki-text">{kiText}</pre>
              {#if kiErgebnis}
                <strong>Antwort:</strong>
                <div class="ki-antwort"><NoteText text={kiErgebnis} /></div>
                <div class="ki-aktionen">
                  <button type="button" onclick={kiVerwerfen}>Verwerfen</button>
                  <button type="button" class="primaer" onclick={kiUebernehmen}>Ins Logbuch übernehmen</button>
                </div>
              {:else}
                <div class="ki-aktionen">
                  <button type="button" onclick={kiVerwerfen}>Abbrechen</button>
                  <button type="button" class="primaer" onclick={kiSenden} disabled={kiLaeuft}>
                    {kiLaeuft ? 'Frage …' : 'Senden'}
                  </button>
                </div>
              {/if}
            </div>
          {/if}
        {/if}
      </section>
    {/if}

    <header class="kopf">
      <h1>{projekt.titel}</h1>
      <div class="kopf-rechts">
        <button type="button" class="schlicht" onclick={() => bearbeitenStarten(projekt)}>Bearbeiten</button>
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
      </div>
    </header>

    {#if bearbeiten}
      <section class="bearbeiten-formular" aria-labelledby="bearbeiten-titel">
        <h2 id="bearbeiten-titel">Projekt bearbeiten</h2>
        <label>
          <span>Titel</span>
          <input bind:value={eTitel} type="text" />
        </label>
        <label>
          <span>Kurs – worum geht es, was ist das Ziel?</span>
          <textarea bind:value={eKurs} rows="3"></textarea>
        </label>
        <div class="feld-paar">
          <label>
            <span>Ruhig für (Tage)</span>
            <input bind:value={eIntervall} type="number" min="1" />
          </label>
          <label>
            <span>Wiedervorlage</span>
            <input bind:value={eWiedervorlage} type="date" />
          </label>
        </div>
        <label>
          <span>Tags, mit Komma getrennt</span>
          <input bind:value={eTags} type="text" placeholder="haus, 2026" />
        </label>
        <div class="aktionen">
          {#if loeschenGefragt}
            <span class="loesch-frage">
              „{projekt.titel}“ mit allen Notizen löschen?
              <button type="button" class="gefahr" onclick={() => projektLoeschen(projekt.titel)} disabled={wirdGespeichert}>
                Ja, löschen
              </button>
              <button type="button" onclick={() => (loeschenGefragt = false)}>Nein</button>
            </span>
          {:else}
            <button type="button" class="gefahr links-weg" onclick={() => (loeschenGefragt = true)}>Löschen</button>
          {/if}
          <button type="button" onclick={() => (bearbeiten = false)}>Abbrechen</button>
          <button type="button" class="primaer" onclick={() => kopfSpeichern(projekt)} disabled={wirdGespeichert}>
            Speichern
          </button>
        </div>
      </section>
    {:else}
      <p class="kurs">{projekt.kurs || 'Noch kein Kurs gesetzt.'}</p>
      <p class="meta">
        {VORLAGEN_LABEL[projekt.vorlage]} · ruhig für {projekt.erwartungsintervall_tage} Tage
        {#if projekt.wiedervorlage}· Wiedervorlage {projekt.wiedervorlage}{/if}
        {#if projekt.tags.length > 0}· {projekt.tags.join(', ')}{/if}
      </p>
    {/if}

    {#if ausstehenderStatus}
      <section class="uebergabe-formular" aria-labelledby="uebergabe-titel">
        <h2 id="uebergabe-titel">Übergabenotiz für Wechsel zu „{STATUS_LABEL[ausstehenderStatus]}“</h2>
        <p class="hinweis">Pflicht, aber billig: kurz festhalten, was der nächste Einstieg wissen muss.</p>
        <textarea rows="4" bind:value={uebergabeText} placeholder="Was ist der Stand? Was ist als Nächstes dran?"></textarea>
        <label class="wiedervorlage">
          <span>Wiedervorlage (optional)</span>
          <input bind:value={wiedervorlage} type="date" />
        </label>
        <div class="uebergabe-aktionen">
          <button type="button" onclick={() => (uebergabeText = 'Nichts Neues, siehe letzte Notiz.')}>
            Nichts Neues, siehe letzte Notiz
          </button>
          <button type="button" class="primaer" onclick={uebergabeAbschicken} disabled={wirdGespeichert}>Speichern</button>
          <button type="button" onclick={() => (ausstehenderStatus = null)}>Abbrechen</button>
        </div>
      </section>
    {/if}
    {#if fehlerText}
      <p class="hinweis fehler">{fehlerText}</p>
    {/if}

    <section class="erfassen" aria-labelledby="erfassen-titel">
      <h2 id="erfassen-titel">Was gibt's?</h2>
      <textarea
        bind:value={notizText}
        onkeydown={aufNotizTaste}
        rows="2"
        placeholder="Ein Satz ins Logbuch. Strg+Enter speichert."
      ></textarea>
      <div class="erfassen-zeile">
        <div class="arten" role="group" aria-label="Art des Eintrags">
          {#each ERFASSBARE_ARTEN as art (art)}
            <button type="button" class:gewaehlt={notizArt === art} onclick={() => (notizArt = art)}>
              {ART_LABEL[art]}
            </button>
          {/each}
        </div>
        <button type="button" class="primaer" onclick={notizSpeichern} disabled={!notizText.trim() || wirdGespeichert}>
          {wirdGespeichert ? 'Speichere …' : 'Eintragen'}
        </button>
      </div>
    </section>

    <section aria-labelledby="offene-faeden-titel">
      <h2 id="offene-faeden-titel">Offene Fäden</h2>
      {#if offeneFaeden.length === 0}
        <p class="hinweis">Keine offenen Fäden.</p>
      {:else}
        <ul class="einfache-liste">
          {#each offeneFaeden as faden (faden.id)}
            <li>
              <button
                type="button"
                class="haken"
                title="Faden abhaken"
                aria-label="Faden abhaken"
                onclick={() => fadenErledigen(faden.id)}
              >
                ✓
              </button>
              <div class="faden-text"><NoteText text={faden.text} /></div>
              <span class="alter">{alterInTagenText(faden.ts)}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section aria-labelledby="referenzen-titel">
      <div class="abschnitt-kopf">
        <h2 id="referenzen-titel">Referenzen</h2>
        <button type="button" class="schlicht" onclick={() => (referenzFormular = !referenzFormular)}>
          {referenzFormular ? 'Abbrechen' : 'Hinzufügen'}
        </button>
      </div>

      {#if referenzFormular}
        <form class="referenz-formular" onsubmit={referenzSpeichern}>
          <select bind:value={rTyp} aria-label="Typ">
            {#each REFERENZ_TYPEN as t (t)}
              <option value={t}>{REFERENZ_TYP_LABEL[t]}</option>
            {/each}
          </select>
          <input
            bind:value={rZiel}
            type="text"
            placeholder="Pfad, URL oder Ort – „Keller, Regal 3, blaue Kiste“"
            aria-label="Ziel"
            required
          />
          <select bind:value={rRolle} aria-label="Rolle">
            {#each REFERENZ_ROLLEN as r (r)}
              <option value={r}>{REFERENZ_ROLLE_LABEL[r]}</option>
            {/each}
          </select>
          <button type="submit" class="primaer" disabled={!rZiel.trim() || wirdGespeichert}>Speichern</button>
        </form>
        <p class="hinweis klein">
          Zwei Adressen kann Lotse selbst lesen: die Abonnement-Adresse eines Kalenders (endet auf
          <code>.ics</code> oder beginnt mit <code>webcal://</code>) erscheint als <em>Was ansteht</em>, und ein Ordner
          mit Git-Remote zu GitHub oder GitLab bringt den Stand der Gegenseite mit.
        </p>
      {/if}

      {#if gegenseite}
        <p class="gegenseite">
          <span class="referenz-typ">{gegenseite.anbieter}</span>
          <button
            type="button"
            class="referenz-ziel oeffnen"
            onclick={() => referenzOeffnen(gegenseite!.url)}
            title="Im Browser öffnen"
          >
            {gegenseite.repo}
          </button>
          {#if gegenseite.herkunft === 'ordner'}<span class="hinweis rolle">aus dem Ordner erkannt</span>{/if}
          <button type="button" class="schlicht" onclick={gegenseiteAbfragen} disabled={gegenseiteLaeuft}>
            {gegenseiteLaeuft ? '…' : 'Stand holen'}
          </button>
        </p>
      {/if}

      {#if referenzen.length === 0}
        <p class="hinweis">Keine Referenzen. Hier gehört hin, wo das Material liegt – auch Regale und Kisten.</p>
      {:else}
        <ul class="einfache-liste">
          {#each referenzen as ref (ref.id)}
            <li>
              <span class="referenz-typ">{REFERENZ_TYP_LABEL[ref.typ]}</span>
              {#if echteDaten && OEFFENBAR.includes(ref.typ)}
                <button
                  type="button"
                  class="referenz-ziel oeffnen"
                  onclick={() => referenzOeffnen(ref.ziel)}
                  title="Im System öffnen"
                >
                  {ref.ziel}
                </button>
              {:else}
                <span class="referenz-ziel">{ref.ziel}</span>
              {/if}
              <span class="hinweis rolle">{REFERENZ_ROLLE_LABEL[ref.rolle]}</span>
              <span class="badge badge--{ref.pruefstatus}">{PRUEFSTATUS_LABEL[ref.pruefstatus]}</span>
              <button type="button" class="schlicht" onclick={() => referenzPruefen(ref.id)} disabled={geprueftWird === ref.id}>
                {geprueftWird === ref.id ? '…' : 'Prüfen'}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    {#if terminQuellen > 0 || terminFehler.length > 0}
      <section aria-labelledby="termine-titel">
        <h2 id="termine-titel">Was ansteht</h2>
        {#each terminFehler as f (f)}
          <p class="hinweis">{f}</p>
        {/each}
        {#if termine.length === 0 && terminFehler.length === 0}
          <p class="hinweis">In den nächsten 90 Tagen steht nichts im Kalender.</p>
        {:else}
          <ul class="einfache-liste">
            {#each termine as t (t.datum + t.titel + (t.uhrzeit ?? ''))}
              <li>
                <span class="termin-datum">{datumText(t.datum)}</span>
                <span class="hinweis rolle">{terminZeit(t)}</span>
                <span class="referenz-ziel">{t.titel}</span>
                {#if t.ort}<span class="hinweis rolle">{t.ort}</span>{/if}
                {#if t.ungenau}
                  <span class="hinweis rolle" title="Die Wiederholungsregel wird nicht ausgerechnet">
                    wiederholt sich
                  </span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {/if}

    {#if echteDaten}
      <section aria-labelledby="deuten-titel">
        <div class="abschnitt-kopf">
          <h2 id="deuten-titel">Datei deuten</h2>
          <button type="button" class="schlicht" onclick={dateiWaehlen} disabled={auszugLaeuft}>
            {auszugLaeuft ? 'Moment …' : 'Datei wählen'}
          </button>
        </div>

        {#if !auszug}
          <p class="hinweis">
            Eine einzelne Datei aufmachen und deuten lassen – PDF, Text, Markdown, CSV oder JSON. Du siehst den
            vollständigen Auszug, bevor etwas gesendet wird. Der Ordner-Beobachter liest weiterhin keine Inhalte.
          </p>
          {#if dateiVorschlaege.length > 0}
            <ul class="einfache-liste">
              {#each dateiVorschlaege as p (p)}
                <li>
                  <button type="button" class="referenz-ziel oeffnen" onclick={() => dateiOeffnen(p)}>{p}</button>
                </li>
              {/each}
            </ul>
          {/if}
        {:else}
          <div class="ki">
            <strong>
              {auszug.name} · {auszug.format_anzeige}{#if auszug.seiten}, {auszug.seiten}
                {auszug.seiten === 1 ? 'Seite' : 'Seiten'}{/if}
            </strong>
            {#if auszug.gekuerzt}
              <p class="hinweis klein">
                Gekürzt: die Datei hat {auszug.zeichen_gesamt.toLocaleString('de-DE')} Zeichen, gesendet wird der
                Anfang.
              </p>
            {/if}
            <strong>Das würde gesendet:</strong>
            <pre class="ki-text">{auszug.text}</pre>
            {#if auszugErgebnis}
              <strong>Antwort:</strong>
              <div class="ki-antwort"><NoteText text={auszugErgebnis} /></div>
              <div class="ki-aktionen">
                <button type="button" onclick={auszugVerwerfen}>Verwerfen</button>
                <button type="button" class="primaer" onclick={auszugUebernehmen}>Ins Logbuch übernehmen</button>
              </div>
            {:else}
              <div class="ki-aktionen">
                <button type="button" onclick={auszugVerwerfen}>Abbrechen</button>
                <button type="button" class="primaer" onclick={auszugSenden} disabled={auszugLaeuft}>
                  {auszugLaeuft ? 'Frage …' : 'Senden'}
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </section>
    {/if}

    <section aria-labelledby="zugaenge-titel">
      <div class="abschnitt-kopf">
        <h2 id="zugaenge-titel">Zugänge</h2>
        <button type="button" class="schlicht" onclick={() => (tresorOffen = true)}>Hinzufügen</button>
      </div>
      <TresorListe eintraege={zugaenge} projekte={[projekt]} />
    </section>

    <section aria-labelledby="logbuch-titel">
      <h2 id="logbuch-titel">Logbuch</h2>
      {#if notizen.length === 0}
        <p class="hinweis">Noch keine Einträge.</p>
      {:else}
        <ul class="logbuch">
          {#each notizen as notiz (notiz.id)}
            <li class:erledigt={notiz.erledigt_am}>
              <div class="logbuch-kopf">
                <span class="badge badge--quelle">{QUELLE_LABEL[notiz.quelle]}</span>
                <span class="hinweis">{ART_LABEL[notiz.art]}</span>
                {#if notiz.erledigt_am}<span class="hinweis">· erledigt</span>{/if}
                <span class="alter" title={datumText(notiz.ts)}>{alterInTagenText(notiz.ts)}</span>
                <button
                  type="button"
                  class="schlicht umsortieren"
                  title="Einem anderen Projekt zuordnen"
                  onclick={() => (verschiebeNotiz = verschiebeNotiz === notiz.id ? null : notiz.id)}
                >
                  ⇢
                </button>
              </div>
              <NoteText text={notiz.text} />
              {#if verschiebeNotiz === notiz.id}
                <div class="verschieben">
                  <label for={`ziel-${notiz.id}`}>Zuordnen zu</label>
                  <select
                    id={`ziel-${notiz.id}`}
                    onchange={(e) => {
                      const ziel = (e.currentTarget as HTMLSelectElement).value;
                      if (ziel) notizVerschieben(notiz.id, ziel);
                    }}
                  >
                    <option value="">Projekt wählen …</option>
                    {#each projekte.filter((p) => p.id !== projekt.id) as p (p.id)}
                      <option value={p.id}>{p.titel}</option>
                    {/each}
                  </select>
                  <button type="button" class="schlicht" onclick={() => (verschiebeNotiz = null)}>Abbrechen</button>
                </div>
              {/if}
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

  .ki-knopf {
    margin-top: 0.8rem;
  }
  .ki {
    margin-top: 0.9rem;
    padding-top: 0.8rem;
    border-top: 1px dashed var(--rahmen);
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.92rem;
  }
  .ki-text {
    background: var(--hintergrund);
    border-radius: 0.4rem;
    padding: 0.6rem 0.8rem;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 14rem;
    overflow-y: auto;
    font-size: 0.85rem;
  }
  .ki-antwort {
    background: var(--hintergrund);
    border-radius: 0.4rem;
    padding: 0.6rem 0.8rem;
  }
  .ki-aktionen {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    margin-top: 0.2rem;
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
  .kopf-rechts {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .kurs {
    font-size: 1.05rem;
    margin: 0.5rem 0 0.3rem;
  }
  .meta {
    font-size: 0.85rem;
    color: var(--text-gedaempft);
    margin: 0 0 1.5rem;
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
  .bearbeiten-formular {
    border: 1px solid var(--akzent);
    border-radius: 0.6rem;
    padding: 1rem 1.2rem;
    margin: 0.8rem 0 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }
  .bearbeiten-formular h2 {
    margin: 0;
  }
  .bearbeiten-formular label,
  .wiedervorlage {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.9rem;
  }
  .bearbeiten-formular label > span,
  .wiedervorlage > span {
    color: var(--text-gedaempft);
  }
  .bearbeiten-formular input,
  .bearbeiten-formular textarea {
    width: 100%;
    font: inherit;
  }
  .feld-paar {
    display: flex;
    gap: 0.8rem;
    flex-wrap: wrap;
  }
  .feld-paar label {
    flex: 1;
    min-width: 9rem;
  }
  .wiedervorlage {
    margin: 0.5rem 0;
    max-width: 12rem;
  }
  .uebergabe-formular textarea {
    width: 100%;
    box-sizing: border-box;
    font: inherit;
    margin: 0.5rem 0;
  }
  .uebergabe-aktionen,
  .aktionen {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .aktionen {
    justify-content: flex-end;
  }

  .erfassen {
    border: 1px solid var(--rahmen);
    border-radius: 0.6rem;
    padding: 0.9rem 1rem;
    background: var(--karten-hintergrund);
  }
  .erfassen h2 {
    margin: 0 0 0.5rem;
  }
  .erfassen textarea {
    width: 100%;
    box-sizing: border-box;
    font: inherit;
    resize: vertical;
  }
  .erfassen-zeile {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.8rem;
    margin-top: 0.6rem;
    flex-wrap: wrap;
  }
  .arten {
    display: flex;
    gap: 0.35rem;
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

  section {
    margin-bottom: 1.75rem;
  }
  h2 {
    font-size: 1rem;
    margin: 0 0 0.6rem;
  }
  .abschnitt-kopf {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.6rem;
  }
  .abschnitt-kopf h2 {
    margin: 0;
  }

  button {
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.4rem 0.85rem;
  }
  button:hover:not(:disabled) {
    border-color: var(--akzent);
  }
  button:disabled {
    opacity: 0.5;
  }
  button.primaer {
    border-color: var(--akzent);
    color: var(--akzent);
    font-weight: 600;
  }
  button.schlicht {
    padding: 0.2rem 0.6rem;
    font-size: 0.8rem;
    background: transparent;
  }
  button.gefahr {
    color: var(--farbe-ueberfaellig);
    border-color: var(--farbe-ueberfaellig);
  }
  .links-weg {
    margin-right: auto;
  }
  .loesch-frage {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
    font-size: 0.88rem;
    color: var(--farbe-ueberfaellig);
    margin-right: auto;
  }
  .umsortieren {
    padding: 0 0.4rem;
    line-height: 1.4;
    color: var(--text-gedaempft);
  }
  .verschieben {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px dashed var(--rahmen);
    font-size: 0.85rem;
  }
  .verschieben label {
    color: var(--text-gedaempft);
  }
  .verschieben select {
    font: inherit;
    color: inherit;
    background: var(--hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.35rem;
    padding: 0.25rem 0.4rem;
  }
  button.oeffnen {
    text-align: left;
    padding: 0;
    border: none;
    background: none;
    color: var(--akzent);
    text-decoration: underline;
    text-underline-offset: 2px;
    font: inherit;
  }
  button.oeffnen:hover {
    text-decoration-thickness: 2px;
  }

  .referenz-formular {
    display: flex;
    gap: 0.4rem;
    margin-bottom: 0.7rem;
    flex-wrap: wrap;
  }
  .referenz-formular input {
    flex: 1;
    min-width: 12rem;
  }
  .referenz-formular select {
    font: inherit;
    color: inherit;
    background: var(--hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.4rem;
    padding: 0.5rem 0.4rem;
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
  .haken {
    align-self: center;
    padding: 0 0.45rem;
    line-height: 1.5;
    color: var(--farbe-ruhig);
    flex: none;
  }
  .faden-text {
    flex: 1;
    min-width: 10rem;
  }
  .klein {
    font-size: 0.85rem;
  }
  .termin-datum {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .gegenseite {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
    margin: 0 0 0.8rem;
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
    overflow-wrap: anywhere;
  }
  .rolle {
    font-size: 0.8rem;
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
  .logbuch li.erledigt {
    opacity: 0.6;
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
