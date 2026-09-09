<script lang="ts">
  // Ordner durchsuchen und beobachten, Export, Abgleich, Geräte, Passwort, Sperren.
  // Alles außer dem Scan-Hinweis ist Sache der Hülle: im Browser gibt es das nicht.
  import { echteDaten, provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';
  import {
    beobachter,
    exportieren,
    forge,
    ki,
    KI_ZIELE,
    konto,
    sync,
    system,
    update,
    type BeobachterStatus,
    type ForgeStatus,
    type KiStatus,
    type KiVerbrauch,
    type GeraetInfo,
    type KontoStatus,
    type SyncErgebnis,
    type SyncStatus,
    type UpdateStand,
  } from '../lib/data/tauri';
  import { datumText } from '../lib/format';
  import { meldungen } from '../lib/meldung.svelte';

  // --- Ordner durchsuchen und beobachten ------------------------------------
  let wurzel = $state('');
  let scanLaeuft = $state(false);
  let scanErgebnis: string | null = $state(null);
  let beoStatus: BeobachterStatus | null = $state(null);

  async function ordnerWaehlen() {
    try {
      const gewaehlt = await system.ordnerWaehlen();
      if (gewaehlt) wurzel = gewaehlt;
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  async function scannen(e: Event) {
    e.preventDefault();
    const pfad = wurzel.trim();
    if (!pfad || scanLaeuft) return;
    scanLaeuft = true;
    scanErgebnis = null;
    try {
      const gefunden = await provider.scan([pfad]);
      scanErgebnis =
        gefunden.length === 0
          ? 'Nichts gefunden. Enthält der Ordner Unterordner mit Erkennungsmarken wie .git, Cargo.toml oder einer README?'
          : `${gefunden.length} ${gefunden.length === 1 ? 'Kandidat' : 'Kandidaten'} gefunden. Sie stehen im Hafen unter „Hafeneinfahrt“.`;
      datenVersion.bump();
    } catch (e2) {
      meldungen.fehler(e2);
    } finally {
      scanLaeuft = false;
    }
  }

  async function beobachtenUmschalten() {
    try {
      if (beoStatus?.laeuft) {
        await beobachter.stoppen();
        meldungen.zeigen('Beobachtung angehalten.');
      } else {
        const wurzeln = wurzel.trim() ? [wurzel.trim()] : (beoStatus?.wurzeln ?? []);
        if (wurzeln.length === 0) {
          meldungen.zeigen('Erst einen Ordner wählen.', 'fehler');
          return;
        }
        await beobachter.starten(wurzeln);
        meldungen.zeigen('Beobachtung läuft.');
      }
      beoStatus = await beobachter.status();
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  // --- Export ---------------------------------------------------------------
  let bundlePass = $state('');
  let exportLaeuft = $state(false);

  async function spiegelSchreiben() {
    exportLaeuft = true;
    try {
      const ziel = await system.ordnerWaehlen();
      if (!ziel) return;
      const anzahl = await exportieren.spiegel(ziel);
      meldungen.zeigen(`${anzahl} ${anzahl === 1 ? 'Projekt' : 'Projekte'} als Markdown geschrieben.`);
    } catch (e) {
      meldungen.fehler(e);
    } finally {
      exportLaeuft = false;
    }
  }

  async function bundleSchreiben(e: Event) {
    e.preventDefault();
    if (bundlePass.length < 8 || exportLaeuft) return;
    exportLaeuft = true;
    try {
      const ziel = await system.dateiWaehlen('lotse-backup.json.age');
      if (!ziel) return;
      const b = await exportieren.bundle(ziel, bundlePass);
      bundlePass = '';
      const rest =
        b.tresor_nicht_lesbar > 0
          ? ` ${b.tresor_nicht_lesbar} Tresor-Einträge waren auf diesem Gerät nicht lesbar.`
          : '';
      meldungen.zeigen(`${b.projekte} Projekte, ${b.notizen} Notizen, ${b.tresor} Zugänge gesichert.${rest}`);
    } catch (e2) {
      meldungen.fehler(e2);
    } finally {
      exportLaeuft = false;
    }
  }

  // --- Remote-Git -----------------------------------------------------------
  let forgeStatus: ForgeStatus | null = $state(null);
  let tresorEintraege: { id: string; titel: string; felder: { name: string }[] }[] = $state([]);
  let forgeEintrag = $state('');
  let forgeFeld = $state('token');
  let forgeEintragGitlab = $state('');
  let forgeFeldGitlab = $state('token');
  let forgeLaeuft = $state(false);

  // Je Hoster eine eigene Bindung: ein GitHub-Token darf nie bei GitLab landen.
  async function forgeVerbinden(anbieter: 'GitHub' | 'GitLab') {
    const eintrag = anbieter === 'GitHub' ? forgeEintrag : forgeEintragGitlab;
    const feld = (anbieter === 'GitHub' ? forgeFeld : forgeFeldGitlab).trim() || 'token';
    try {
      await forge.tokenSetzen(anbieter, eintrag, feld);
      forgeStatus = await forge.status();
      meldungen.zeigen(eintrag ? `Token für ${anbieter} hinterlegt.` : `Token-Bindung für ${anbieter} gelöst.`);
    } catch (e2) {
      meldungen.fehler(e2);
    }
  }

  async function forgeAutoUmschalten(an: boolean) {
    try {
      await forge.autoSetzen(an);
      forgeStatus = await forge.status();
      meldungen.zeigen(
        an
          ? 'Die Gegenseite wird vom Ordner-Beobachter mit abgefragt, höchstens alle 30 Minuten.'
          : 'Die Gegenseite wird nur noch auf Knopfdruck abgefragt.',
      );
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  async function forgeAbfragen() {
    forgeLaeuft = true;
    try {
      const r = await forge.abfragen();
      datenVersion.bump();
      for (const f of r.fehler) meldungen.zeigen(f, 'fehler');
      if (r.fehler.length === 0 || r.abgefragt > 0) {
        meldungen.zeigen(
          r.notizen > 0
            ? `${r.abgefragt} Repos abgefragt, ${r.notizen} ${r.notizen === 1 ? 'Notiz' : 'Notizen'} geschrieben.`
            : `${r.abgefragt} Repos abgefragt, nichts Neues zu melden.`,
        );
      }
    } catch (e) {
      meldungen.fehler(e);
    } finally {
      forgeLaeuft = false;
    }
  }

  // --- Was die KI bisher gesendet hat ---------------------------------------
  let kiVerbrauch: KiVerbrauch | null = $state(null);

  async function kiVerbrauchLoeschen() {
    try {
      await ki.verbrauchLoeschen();
      kiVerbrauch = await ki.verbrauch();
      autoLockMinuten = await konto.autoLock();
      meldungen.zeigen('Zähler und Protokoll gelöscht.');
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  // --- Version und Updates --------------------------------------------------
  let updateStand: UpdateStand | null = $state(null);
  let updateLaeuft = $state(false);

  // Ohne eigenes Fangen bliebe eine abgelehnte Zusage unbeachtet: der Klick täte
  // scheinbar nichts.
  async function oeffnen(ziel: string) {
    try {
      await system.oeffnen(ziel);
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  async function updatePruefen() {
    updateLaeuft = true;
    try {
      updateStand = await update.pruefen(true);
      if (!updateStand.neu) meldungen.zeigen('Diese Version ist die neueste.');
    } catch (e) {
      meldungen.fehler(e);
    } finally {
      updateLaeuft = false;
    }
  }

  async function updateAutomatischUmschalten(an: boolean) {
    try {
      await update.automatischSetzen(an);
      updateStand = await update.pruefen(false);
      kiVerbrauch = await ki.verbrauch();
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  // --- KI ------------------------------------------------------------------
  let kiStatus: KiStatus | null = $state(null);
  let kiUrl: string = $state(KI_ZIELE[0].url);
  let kiModell = $state('');
  let kiModelle: string[] = $state([]);
  let kiEintrag = $state('');
  let kiFeld = $state('schluessel');
  let kiLaeuft = $state(false);

  const kiIstLokal = $derived(kiUrl.startsWith('http://localhost') || kiUrl.startsWith('http://127.0.0.1'));

  async function kiModelleLaden() {
    kiLaeuft = true;
    try {
      kiModelle = await ki.modelle(kiUrl);
      if (kiModelle.length > 0 && !kiModelle.includes(kiModell)) kiModell = kiModelle[0];
      meldungen.zeigen(`${kiModelle.length} Modelle gefunden.`);
    } catch (e) {
      kiModelle = [];
      meldungen.fehler(e);
    } finally {
      kiLaeuft = false;
    }
  }

  async function kiSpeichern(e: Event) {
    e.preventDefault();
    try {
      await ki.zielSetzen(kiUrl, kiModell, kiEintrag, kiFeld.trim() || 'schluessel');
      kiStatus = await ki.status();
      meldungen.zeigen('Ziel gemerkt.');
    } catch (e2) {
      meldungen.fehler(e2);
    }
  }

  // --- Konto, Abgleich, Geräte ---------------------------------------------
  let kontoStatus: KontoStatus | null = $state(null);
  let syncStatus: SyncStatus | null = $state(null);
  let syncLaeuft = $state(false);
  let letztesErgebnis: SyncErgebnis | null = $state(null);
  let geraete: GeraetInfo[] = $state([]);
  let widerrufKandidat: string | null = $state(null);

  let registrierenOffen = $state(false);
  let url = $state('');
  let email = $state('');
  let code = $state('');

  async function statusLaden() {
    if (!echteDaten) return;
    try {
      kontoStatus = await konto.status();
      syncStatus = await sync.status();
      beoStatus = await beobachter.status();
      geraete = syncStatus.eingerichtet ? await sync.geraete() : [];
      forgeStatus = await forge.status();
      forgeEintrag = forgeStatus.token_eintrag ?? '';
      forgeFeld = forgeStatus.token_feld ?? 'token';
      forgeEintragGitlab = forgeStatus.token_eintrag_gitlab ?? '';
      forgeFeldGitlab = forgeStatus.token_feld_gitlab ?? 'token';
      updateStand = await update.pruefen(false);
      tresorEintraege = await provider.listAllVaultEntries();
      kiStatus = await ki.status();
      kiUrl = kiStatus.basis_url;
      kiModell = kiStatus.modell;
      kiEintrag = kiStatus.schluessel_eintrag ?? '';
      kiFeld = kiStatus.schluessel_feld ?? 'schluessel';
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  $effect(() => {
    datenVersion.wert;
    void statusLaden();
  });

  async function jetztAbgleichen() {
    syncLaeuft = true;
    letztesErgebnis = null;
    try {
      letztesErgebnis = await sync.jetzt();
      await statusLaden();
      datenVersion.bump();
    } catch (e) {
      meldungen.fehler(e);
    } finally {
      syncLaeuft = false;
    }
  }

  async function registrieren(e: Event) {
    e.preventDefault();
    if (syncLaeuft) return;
    syncLaeuft = true;
    try {
      await sync.registrieren(url.trim(), email.trim(), code.trim());
      code = '';
      registrierenOffen = false;
      await statusLaden();
      meldungen.zeigen('Abgleich eingerichtet.');
    } catch (e2) {
      meldungen.fehler(e2);
    } finally {
      syncLaeuft = false;
    }
  }

  async function widerrufen(id: string) {
    try {
      await sync.geraetWiderrufen(id);
      widerrufKandidat = null;
      await statusLaden();
      meldungen.zeigen('Gerät widerrufen.');
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  // --- Passwort wechseln ----------------------------------------------------
  let pwOffen = $state(false);
  let pwAlt = $state('');
  let pwNeu = $state('');
  let pwCode = $state('');
  let pwLaeuft = $state(false);
  let neuerCode: string | null = $state(null);

  async function passwortAendern(e: Event) {
    e.preventDefault();
    if (pwNeu.length < 12 || pwLaeuft) return;
    pwLaeuft = true;
    try {
      const frisch = await konto.passwortAendern(pwAlt, pwNeu, pwCode.trim() || undefined);
      pwAlt = '';
      pwNeu = '';
      pwCode = '';
      pwOffen = false;
      if (frisch) {
        neuerCode = frisch;
      } else {
        meldungen.zeigen('Passwort geändert. Dein Wiederherstellungscode gilt weiter.');
      }
    } catch (e2) {
      meldungen.fehler(e2);
    } finally {
      pwLaeuft = false;
    }
  }

  let autoLockMinuten = $state(15);

  async function autoLockSetzen(minuten: number) {
    try {
      await konto.autoLockSetzen(minuten);
      autoLockMinuten = minuten;
      meldungen.zeigen(minuten === 0 ? 'Lotse sperrt nicht mehr von selbst.' : `Sperrt nach ${minuten} Minuten Ruhe.`);
    } catch (e) {
      meldungen.fehler(e);
    }
  }

  async function sperren() {
    await konto.sperren();
    // Der Entsperr-Bildschirm hängt am Startzustand der App.
    window.location.reload();
  }
</script>

<svelte:head><title>Einstellungen · Lotse</title></svelte:head>

<h1>Einstellungen</h1>

<section aria-labelledby="ordner-titel">
  <h2 id="ordner-titel">Ordner</h2>
  <p class="hinweis">
    Lotse sucht in den Unterordnern nach Erkennungsmarken und schlägt gefundene Projekte in der Hafeneinfahrt vor.
    Übernommen wird nichts von allein, und Dateiinhalte liest Lotse dabei nicht.
  </p>
  {#if echteDaten}
    <form class="zeile" onsubmit={scannen}>
      <input
        bind:value={wurzel}
        type="text"
        placeholder={beoStatus?.wurzeln[0] ?? 'Noch kein Ordner gewählt'}
        aria-label="Wurzelordner"
      />
      <button type="button" onclick={ordnerWaehlen}>Ordner wählen …</button>
      <button type="submit" class="primaer" disabled={!wurzel.trim() || scanLaeuft}>
        {scanLaeuft ? 'Suche …' : 'Einmal durchsuchen'}
      </button>
    </form>
    {#if scanErgebnis}
      <p class="ergebnis">{scanErgebnis} <a href="#/">Zum Hafen</a></p>
    {/if}

    <div class="beobachter">
      <div>
        <strong>Laufend beobachten</strong>
        <p class="hinweis">
          Schreibt geänderte Dateien und neue Commits als eine verdichtete Zeile pro Projekt und Tag ins Logbuch.
          Nur Namen und Zeiten, keine Inhalte; <code>.env</code>, Schlüssel und Zertifikate stehen auf einer festen
          Ausschlussliste.
        </p>
        {#if beoStatus && beoStatus.wurzeln.length > 0}
          <p class="mono klein">{beoStatus.wurzeln.join(' · ')}</p>
        {/if}
      </div>
      <button type="button" class:primaer={!beoStatus?.laeuft} onclick={beobachtenUmschalten}>
        {beoStatus?.laeuft ? 'Anhalten' : 'Starten'}
      </button>
    </div>
    {#if beoStatus?.laeuft}
      <p class="ergebnis">Läuft. Neue Einträge erscheinen von selbst im Logbuch.</p>
    {/if}
  {:else}
    <p class="nur-desktop">Nur in der Desktop-App: der Browser hat keinen Zugriff auf deine Ordner.</p>
  {/if}
</section>

<section aria-labelledby="export-titel">
  <h2 id="export-titel">Export</h2>
  <p class="hinweis">
    Wenn Lotse morgen weg ist, kommst du trotzdem an alles. Der Spiegel ist ein Ordner mit Markdown für grep und
    Obsidian; das Bundle enthält den ganzen Bestand samt Tresor und lässt sich mit <code>age</code> auch ohne Lotse
    öffnen.
  </p>
  {#if echteDaten}
    <div class="zeile">
      <button type="button" onclick={spiegelSchreiben} disabled={exportLaeuft}>Klartext-Spiegel schreiben …</button>
    </div>
    <form class="zeile" onsubmit={bundleSchreiben}>
      <input
        bind:value={bundlePass}
        type="password"
        placeholder="Passphrase für das Bundle (mind. 8 Zeichen)"
        aria-label="Passphrase"
        autocomplete="new-password"
      />
      <button type="submit" disabled={bundlePass.length < 8 || exportLaeuft}>Verschlüsseltes Bundle …</button>
    </form>
    <p class="hinweis klein">
      Öffnen ohne Lotse: <code>age -d -o bundle.json backup.json.age</code>
    </p>
  {:else}
    <p class="nur-desktop">Nur in der Desktop-App.</p>
  {/if}
</section>

<section aria-labelledby="ki-titel">
  <h2 id="ki-titel">KI-Verdichtung</h2>
  <p class="hinweis">
    Fasst auf Wunsch zusammen, was seit dem letzten Besuch passiert ist – auf der Projektseite, pro Aufruf und nur
    auf Klick. <strong>Vor dem Senden siehst du den vollständigen Text</strong>, der das Gerät verlassen würde. Der
    Tresor ist für die KI unerreichbar, und das Ergebnis ist ein Vorschlag: es landet nur im Logbuch, wenn du es
    übernimmst.
  </p>
  {#if !echteDaten}
    <p class="nur-desktop">Nur in der Desktop-App.</p>
  {:else}
    {#if kiStatus?.ollama_da}
      <p class="ergebnis">Ollama läuft auf diesem Rechner. Damit bleibt alles lokal – kein Schlüssel, kein Konto.</p>
    {:else}
      <p class="hinweis klein">
        Kein Ollama gefunden. Entweder <code>ollama pull llama3.2</code> ausführen, oder unten ein gehostetes Ziel
        mit Schlüssel wählen.
      </p>
    {/if}

    <form class="formular breit" onsubmit={kiSpeichern}>
      <label>
        <span>Ziel</span>
        <select bind:value={kiUrl}>
          {#each KI_ZIELE as z (z.url)}
            <option value={z.url}>{z.name}</option>
          {/each}
        </select>
      </label>

      {#if !kiIstLokal}
        <label>
          <span>Schlüssel aus dem Tresor</span>
          <select bind:value={kiEintrag}>
            <option value="">Kein Eintrag gewählt</option>
            {#each tresorEintraege as t (t.id)}
              <option value={t.id}>{t.titel}</option>
            {/each}
          </select>
          <small class="hinweis">
            Den API-Schlüssel als Tresor-Eintrag anlegen und hier wählen. Er wird nur beim Aufruf gelesen.
          </small>
        </label>
        <label>
          <span>Feldname im Eintrag</span>
          <input bind:value={kiFeld} type="text" placeholder="schluessel" />
        </label>
      {/if}

      <label>
        <span>Modell</span>
        <div class="zeile">
          {#if kiModelle.length > 0}
            <select bind:value={kiModell}>
              {#each kiModelle as m (m)}
                <option value={m}>{m}</option>
              {/each}
            </select>
          {:else}
            <input bind:value={kiModell} type="text" placeholder="z. B. llama3.2" />
          {/if}
          <button type="button" onclick={kiModelleLaden} disabled={kiLaeuft}>
            {kiLaeuft ? 'Frage …' : 'Modelle abrufen'}
          </button>
        </div>
      </label>

      <div class="aktionen">
        <button type="submit" class="primaer" disabled={!kiModell.trim()}>Merken</button>
      </div>
    </form>

    {#if kiVerbrauch}
      <h3>Was bisher gesendet wurde</h3>
      {#if kiVerbrauch.anfragen === 0}
        <p class="hinweis klein">Noch nichts. Hier steht später, was wann an welches Ziel ging.</p>
      {:else}
        <p class="hinweis klein">
          {kiVerbrauch.anfragen}
          {kiVerbrauch.anfragen === 1 ? 'Anfrage' : 'Anfragen'}, {kiVerbrauch.eingabe_token.toLocaleString('de-DE')} Token
          hin und {kiVerbrauch.ausgabe_token.toLocaleString('de-DE')} zurück. Der gesendete Text selbst wird nicht
          aufbewahrt – nur, dass und wohin gesendet wurde.
        </p>
        <ul class="protokoll">
          {#each kiVerbrauch.protokoll.slice(0, 8) as p, i (p.ts + '-' + i)}
            <li>
              <span class="termin-datum">{new Date(p.ts).toLocaleString('de-DE')}</span>
              <span class="hinweis rolle">{p.ziel} · {p.modell}</span>
              <span class="hinweis rolle">{p.zeichen.toLocaleString('de-DE')} Zeichen</span>
            </li>
          {/each}
        </ul>
        <button type="button" onclick={kiVerbrauchLoeschen}>Zähler und Protokoll löschen</button>
      {/if}
    {/if}
  {/if}
</section>

<section aria-labelledby="forge-titel">
  <h2 id="forge-titel">GitHub und GitLab</h2>
  <p class="hinweis">
    Holt zu Projekten mit einem Repo auf der Gegenseite, was der lokale Git-Log nicht weiß: offene Pull bzw. Merge
    Requests, die Zahl offener Issues und ob der Prüflauf rot ist. Das landet als eine verdichtete Zeile im Logbuch –
    Issues werden <strong>nicht</strong> zu offenen Fäden, denn Tickets und Backlogs sind erklärtes Nicht-Ziel.
  </p>
  {#if !echteDaten}
    <p class="nur-desktop">Nur in der Desktop-App.</p>
  {:else if !forgeStatus}
    <p class="hinweis">Lade Status…</p>
  {:else if forgeStatus.projekte.length === 0}
    <p class="hinweis">
      Kein Projekt zeigt bisher auf ein Repo bei GitHub oder GitLab. Am wenigsten Arbeit macht eine Ordner-Referenz
      auf einen geklonten Arbeitsordner: Lotse liest dessen Git-Remote selbst. Sonst eine Referenz vom Typ
      <em>URL</em> mit <code>https://github.com/name/repo</code> oder <code>https://gitlab.com/gruppe/repo</code>.
    </p>
  {:else}
    <ul class="repos">
      {#each forgeStatus.projekte as p (p.projekt_id)}
        <li>
          <a href={`#/projekt/${p.projekt_id}`}>{p.titel}</a>
          <span class="hinweis klein">{p.anbieter}: {p.repo}</span>
          {#if p.herkunft === 'ordner'}<span class="hinweis klein">– aus dem Ordner erkannt</span>{/if}
        </li>
      {/each}
    </ul>
    <form class="zeile" onsubmit={(e) => { e.preventDefault(); forgeVerbinden('GitHub'); }}>
      <select bind:value={forgeEintrag} aria-label="Tresor-Eintrag mit dem GitHub-Token">
        <option value="">GitHub ohne Token (nur öffentliche Repos, kleines Kontingent)</option>
        {#each tresorEintraege as t (t.id)}
          <option value={t.id}>GitHub: {t.titel}</option>
        {/each}
      </select>
      <input bind:value={forgeFeld} type="text" placeholder="Feldname" aria-label="Feldname" class="schmal" />
      <button type="submit">Merken</button>
    </form>
    <form class="zeile" onsubmit={(e) => { e.preventDefault(); forgeVerbinden('GitLab'); }}>
      <select bind:value={forgeEintragGitlab} aria-label="Tresor-Eintrag mit dem GitLab-Token">
        <option value="">GitLab ohne Token (nur öffentliche Repos, kleines Kontingent)</option>
        {#each tresorEintraege as t (t.id)}
          <option value={t.id}>GitLab: {t.titel}</option>
        {/each}
      </select>
      <input bind:value={forgeFeldGitlab} type="text" placeholder="Feldname" aria-label="Feldname" class="schmal" />
      <button type="submit">Merken</button>
    </form>
    <p class="hinweis klein">
      Für private Repos und ein größeres Kontingent: einen Token als Tresor-Eintrag anlegen und hier wählen – je
      Hoster getrennt, damit ein GitHub-Token nie bei GitLab landet. Lotse liest ihn nur beim Abfragen.
    </p>
    <label class="schalter">
      <input
        type="checkbox"
        checked={forgeStatus.auto}
        onchange={(e) => forgeAutoUmschalten(e.currentTarget.checked)}
      />
      Mit dem Ordner-Beobachter mitlaufen lassen
    </label>
    <p class="hinweis klein">
      Dann fragt Lotse die Gegenseite selbsttätig ab, solange der Beobachter läuft – höchstens alle 30 Minuten,
      damit das Kontingent reicht.
      {#if forgeStatus.zuletzt}
        Zuletzt versucht: {new Date(forgeStatus.zuletzt).toLocaleString('de-DE')}.
      {/if}
    </p>
    <button type="button" class="primaer" onclick={forgeAbfragen} disabled={forgeLaeuft}>
      {forgeLaeuft ? 'Frage ab …' : 'Jetzt abfragen'}
    </button>
  {/if}
</section>

<section aria-labelledby="kalender-titel">
  <h2 id="kalender-titel">Kalender</h2>
  <p class="hinweis">
    Lotse liest Kalender, es führt keinen. Leg auf einer Projektseite unter <em>Referenzen</em> eine Referenz vom Typ
    <em>URL</em> mit der Abonnement-Adresse an (endet auf <code>.ics</code> oder beginnt mit <code>webcal://</code>);
    jeder gängige Dienst gibt so eine Adresse aus. Auf der Projektseite steht dann unter <em>Was ansteht</em>, was in
    den nächsten 90 Tagen kommt. Termine wandern nicht ins Logbuch – sie bleiben dort, wo sie gepflegt werden.
  </p>
  <p class="hinweis klein">
    Geholt wird der Kalender, wenn du die Projektseite öffnest, und danach höchstens alle 15 Minuten neu.
    Uhrzeiten zeigt Lotse so, wie sie im Kalender stehen; ohne Zeitzonendatenbank wäre jede Umrechnung geraten.
    Wiederholungen rechnet es aus, solange die Regel einfach ist (täglich, wöchentlich, monatlich, jährlich, auch mit
    Wochentagen). Bei komplizierteren Regeln – „zweiter Montag im Monat“ – steht nur der erste Termin und der Hinweis,
    dass er sich wiederholt.
  </p>
</section>

<section aria-labelledby="sync-titel">
  <h2 id="sync-titel">Abgleich zwischen Geräten</h2>
  {#if !echteDaten}
    <p class="nur-desktop">Nur in der Desktop-App.</p>
  {:else if !syncStatus}
    <p class="hinweis">Lade Status…</p>
  {:else if !syncStatus.eingerichtet}
    <p class="hinweis">
      Noch kein Abgleich eingerichtet. Ohne ihn bleiben alle Daten auf diesem Rechner – das ist ein gültiger Zustand,
      kein Mangel.
    </p>
    {#if registrierenOffen}
      <form class="formular" onsubmit={registrieren}>
        <label>
          <span>Adresse des Dienstes</span>
          <input bind:value={url} type="url" placeholder="https://sync.example.org" required />
        </label>
        <label>
          <span>E-Mail</span>
          <input bind:value={email} type="email" placeholder="du@example.org" required />
        </label>
        <label>
          <span>Wiederherstellungscode</span>
          <input bind:value={code} type="text" placeholder="5YD6-AR19-…" required autocomplete="off" />
          <small class="hinweis">Der Code von der Einrichtung. Er bleibt auf diesem Gerät.</small>
        </label>
        <div class="aktionen">
          <button type="button" onclick={() => (registrierenOffen = false)}>Abbrechen</button>
          <button type="submit" class="primaer" disabled={syncLaeuft}>{syncLaeuft ? 'Melde an …' : 'Konto anlegen'}</button>
        </div>
      </form>
    {:else}
      <button type="button" onclick={() => (registrierenOffen = true)}>Abgleich einrichten</button>
    {/if}
  {:else}
    <dl class="werte">
      <dt>Dienst</dt>
      <dd>{syncStatus.url ?? '—'}</dd>
      <dt>Konto</dt>
      <dd>{syncStatus.email ?? '—'}</dd>
      <dt>Ausstehend</dt>
      <dd>{syncStatus.ausstehend} {syncStatus.ausstehend === 1 ? 'Änderung' : 'Änderungen'}</dd>
      <dt>Zuletzt</dt>
      <dd>{syncStatus.last_sync_ms ? datumText(new Date(syncStatus.last_sync_ms).toISOString()) : 'noch nie'}</dd>
    </dl>
    <button type="button" class="primaer" onclick={jetztAbgleichen} disabled={syncLaeuft}>
      {syncLaeuft ? 'Gleiche ab …' : 'Jetzt abgleichen'}
    </button>
    {#if letztesErgebnis}
      <p class="ergebnis">
        {letztesErgebnis.gepusht} gesendet, {letztesErgebnis.uebernommen} übernommen, {letztesErgebnis.verworfen} verworfen,
        {letztesErgebnis.konflikte}
        {letztesErgebnis.konflikte === 1 ? 'Konflikt' : 'Konflikte'}.
      </p>
    {/if}

    {#if geraete.length > 0}
      <h3>Geräte</h3>
      <ul class="geraete">
        {#each geraete as g (g.id)}
          <li>
            <div>
              <strong>{g.name}</strong>
              <span class="hinweis klein">{g.platform} · zuletzt {datumText(new Date(g.last_seen_at).toISOString())}</span>
            </div>
            {#if widerrufKandidat === g.id}
              <span class="frage">
                Zugang entziehen?
                <button type="button" class="gefahr" onclick={() => widerrufen(g.id)}>Ja</button>
                <button type="button" onclick={() => (widerrufKandidat = null)}>Nein</button>
              </span>
            {:else}
              <button type="button" onclick={() => (widerrufKandidat = g.id)}>Widerrufen</button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</section>

{#if echteDaten}
  <section aria-labelledby="sicherheit-titel">
    <h2 id="sicherheit-titel">Sicherheit</h2>

    {#if neuerCode}
      <div class="code-anzeige">
        <strong>Dein neuer Wiederherstellungscode</strong>
        <p class="hinweis">
          Der bisherige gilt nicht mehr. Dieser hier wird genau einmal angezeigt – jetzt in den Passwortmanager.
        </p>
        <code class="code">{neuerCode}</code>
        <button type="button" class="primaer" onclick={() => (neuerCode = null)}>Habe ich gesichert</button>
      </div>
    {/if}

    {#if pwOffen}
      <form class="formular" onsubmit={passwortAendern}>
        <label>
          <span>Bisheriges Passwort</span>
          <input bind:value={pwAlt} type="password" required autocomplete="current-password" />
        </label>
        <label>
          <span>Neues Passwort (mind. 12 Zeichen)</span>
          <input bind:value={pwNeu} type="password" required minlength="12" autocomplete="new-password" />
        </label>
        <label>
          <span>Wiederherstellungscode <em>(optional)</em></span>
          <input bind:value={pwCode} type="text" placeholder="5YD6-AR19-…" autocomplete="off" />
          <small class="hinweis">
            Der Code hängt am Passwort und wird beim Wechsel neu verankert. Gibst du ihn an, gilt er weiter. Lässt du
            das Feld leer, entsteht ein neuer und der bisherige wird ungültig.
          </small>
        </label>
        <div class="aktionen">
          <button type="button" onclick={() => (pwOffen = false)}>Abbrechen</button>
          <button type="submit" class="primaer" disabled={pwNeu.length < 12 || pwLaeuft}>
            {pwLaeuft ? 'Wechsle …' : 'Passwort ändern'}
          </button>
        </div>
      </form>
    {:else}
      <button type="button" onclick={() => (pwOffen = true)}>Master-Passwort ändern</button>
    {/if}
  </section>

  <section aria-labelledby="update-titel">
  <h2 id="update-titel">Version und Updates</h2>
  {#if !echteDaten}
    <p class="nur-desktop">Nur in der Desktop-App.</p>
  {:else if !updateStand}
    <p class="hinweis">Lade Status…</p>
  {:else}
    <dl class="werte">
      <dt>Diese Version</dt>
      <dd>{updateStand.laufend}</dd>
      {#if updateStand.zuletzt}
        <dt>Zuletzt nachgesehen</dt>
        <dd>{new Date(updateStand.zuletzt).toLocaleString('de-DE')}</dd>
      {/if}
    </dl>
    {#if updateStand.neu}
      <p class="hinweis">
        <strong>Version {updateStand.neu} ist da{updateStand.vorab ? ' (Vorabversion)' : ''}.</strong>
        {#if updateStand.datei}
          Für dieses System: <code>{updateStand.datei}</code>.
        {/if}
      </p>
      <div class="zeile">
        {#if updateStand.datei_url}
          <button type="button" class="primaer" onclick={() => oeffnen(updateStand!.datei_url!)}>
            Herunterladen
          </button>
        {/if}
        {#if updateStand.seite}
          <button type="button" onclick={() => oeffnen(updateStand!.seite!)}>Was ist neu</button>
        {/if}
      </div>
    {:else}
      <p class="hinweis">Keine neuere Version bekannt.</p>
    {/if}
    <label class="schalter">
      <input
        type="checkbox"
        checked={updateStand.automatisch}
        onchange={(e) => updateAutomatischUmschalten(e.currentTarget.checked)}
      />
      Beim Entsperren nachsehen, höchstens einmal am Tag
    </label>
    <p class="hinweis klein">
      Nachsehen heißt: eine Anfrage an die Veröffentlichungen dieses Projekts auf GitHub. GitHub sieht dabei die
      IP-Adresse, sonst nichts – kein Konto, keine Kennung, keine Nutzungsdaten. Heruntergeladen und installiert wird
      nichts von allein: die Installer sind unsigniert, deshalb bleibt der letzte Schritt bewusst deiner.
    </p>
    <button type="button" onclick={updatePruefen} disabled={updateLaeuft}>
      {updateLaeuft ? 'Sehe nach …' : 'Jetzt nachsehen'}
    </button>
  {/if}
</section>

<section aria-labelledby="geraet-titel">
    <h2 id="geraet-titel">Dieses Gerät</h2>
    {#if kontoStatus}
      <dl class="werte">
        <dt>Name</dt>
        <dd>{kontoStatus.geraet ?? '—'}</dd>
        <dt>Datenordner</dt>
        <dd><code>{kontoStatus.home}</code></dd>
      </dl>
    {/if}
    <label class="zeile">
      <span>Von selbst sperren nach</span>
      <select value={autoLockMinuten} onchange={(e) => autoLockSetzen(Number(e.currentTarget.value))}>
        <option value={5}>5 Minuten</option>
        <option value={15}>15 Minuten</option>
        <option value={30}>30 Minuten</option>
        <option value={60}>60 Minuten</option>
        <option value={0}>nie</option>
      </select>
    </label>
    <p class="hinweis klein">
      Dreißig Sekunden vorher meldet sich Lotse; jede Tasten- oder Mausbewegung stellt den Zähler zurück. Sperren
      beendet auch den Ordner-Beobachter – gesperrt heißt gesperrt, der Schlüssel verlässt den Speicher.
    </p>
    <button type="button" onclick={sperren}>Jetzt sperren</button>
  </section>
{/if}

<style>
  h1 {
    font-size: 1.5rem;
    margin: 0 0 1.5rem;
  }
  section {
    margin-bottom: 2rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--rahmen);
  }
  section:last-child {
    border-bottom: none;
  }
  h2 {
    font-size: 1.05rem;
    margin: 0 0 0.5rem;
  }
  h3 {
    font-size: 0.95rem;
    margin: 1.4rem 0 0.5rem;
  }
  .hinweis {
    color: var(--text-gedaempft);
    max-width: 46rem;
  }
  .klein {
    font-size: 0.85rem;
  }
  .mono {
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    overflow-wrap: anywhere;
  }
  .nur-desktop {
    color: var(--text-gedaempft);
    border: 1px dashed var(--rahmen);
    border-radius: 0.5rem;
    padding: 0.6rem 0.8rem;
    font-size: 0.9rem;
  }
  .zeile {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.8rem;
    flex-wrap: wrap;
  }
  .zeile input {
    flex: 1;
    min-width: 15rem;
  }
  .protokoll {
    list-style: none;
    margin: 0.6rem 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
  }
  .protokoll li {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }
  .termin-datum {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .rolle {
    font-size: 0.85rem;
  }
  .schalter {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
    font-size: 0.95rem;
  }
  .schalter input {
    width: auto;
    margin: 0;
  }
  .beobachter {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    margin-top: 1.2rem;
    padding: 0.9rem 1rem;
    border: 1px solid var(--rahmen);
    border-radius: 0.6rem;
    background: var(--karten-hintergrund);
  }
  .beobachter p {
    margin: 0.3rem 0 0;
    font-size: 0.9rem;
  }
  .formular {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
    margin-top: 0.8rem;
    max-width: 26rem;
  }
  .formular label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.9rem;
  }
  .formular label > span {
    color: var(--text-gedaempft);
  }
  .formular label em {
    font-style: normal;
    opacity: 0.7;
  }
  .formular small {
    font-size: 0.8rem;
  }
  .aktionen {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
  button {
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.45rem 0.9rem;
  }
  button:hover:not(:disabled) {
    border-color: var(--akzent);
  }
  button.primaer {
    border-color: var(--akzent);
    color: var(--akzent);
    font-weight: 600;
  }
  button.gefahr {
    border-color: var(--farbe-ueberfaellig);
    color: var(--farbe-ueberfaellig);
  }
  button:disabled {
    opacity: 0.5;
  }
  .werte {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.2rem 1rem;
    margin: 0.8rem 0;
    font-size: 0.92rem;
  }
  .werte dt {
    color: var(--text-gedaempft);
  }
  .werte dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .geraete {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .geraete li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    padding: 0.6rem 0.85rem;
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    background: var(--karten-hintergrund);
  }
  .geraete li > div {
    display: flex;
    flex-direction: column;
  }
  .frage {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
    color: var(--farbe-ueberfaellig);
  }
  .code-anzeige {
    border: 1px solid var(--farbe-auffaellig);
    border-radius: 0.6rem;
    padding: 1rem 1.2rem;
    margin-bottom: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    align-items: flex-start;
  }
  .code {
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 1.05rem;
    letter-spacing: 0.05em;
    background: var(--hintergrund);
    border-radius: 0.4rem;
    padding: 0.6rem 0.8rem;
    overflow-wrap: anywhere;
  }
  .breit {
    max-width: 34rem;
  }
  .repos {
    list-style: none;
    margin: 0.8rem 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.92rem;
  }
  select {
    font: inherit;
    color: inherit;
    background: var(--hintergrund);
    border: 1px solid var(--rahmen);
    border-radius: 0.4rem;
    padding: 0.45rem 0.6rem;
    flex: 1;
    min-width: 14rem;
  }
  .schmal {
    flex: 0 0 9rem;
    min-width: 0;
  }
  .ergebnis {
    margin-top: 0.7rem;
    color: var(--farbe-ruhig);
    font-size: 0.92rem;
  }
</style>
