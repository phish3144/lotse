<script lang="ts">
  // Ordner durchsuchen, Abgleich zwischen Geräten, Konto sperren.
  // Sync und Konto sind Belange der Hülle: im Browser gibt es sie (noch) nicht.
  import { echteDaten, provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';
  import { konto, sync, type KontoStatus, type SyncErgebnis, type SyncStatus } from '../lib/data/tauri';
  import { datumText } from '../lib/format';

  // --- Ordner durchsuchen ---------------------------------------------------
  let wurzel = $state('');
  let scanLaeuft = $state(false);
  let scanErgebnis: string | null = $state(null);
  let scanFehler: string | null = $state(null);

  async function scannen(e: Event) {
    e.preventDefault();
    const pfad = wurzel.trim();
    if (!pfad || scanLaeuft) return;
    scanLaeuft = true;
    scanErgebnis = null;
    scanFehler = null;
    try {
      const gefunden = await provider.scan([pfad]);
      scanErgebnis =
        gefunden.length === 0
          ? 'Nichts gefunden. Enthält der Ordner Unterordner mit Erkennungsmarken wie .git, Cargo.toml oder einer README?'
          : `${gefunden.length} ${gefunden.length === 1 ? 'Kandidat' : 'Kandidaten'} gefunden. Sie stehen im Hafen unter „Hafeneinfahrt“.`;
      datenVersion.bump();
    } catch (e2) {
      scanFehler = e2 instanceof Error ? e2.message : String(e2);
    } finally {
      scanLaeuft = false;
    }
  }

  // --- Konto und Abgleich ---------------------------------------------------
  let kontoStatus: KontoStatus | null = $state(null);
  let syncStatus: SyncStatus | null = $state(null);
  let syncFehler: string | null = $state(null);
  let syncLaeuft = $state(false);
  let letztesErgebnis: SyncErgebnis | null = $state(null);

  let registrierenOffen = $state(false);
  let url = $state('');
  let email = $state('');
  let code = $state('');

  async function statusLaden() {
    if (!echteDaten) return;
    try {
      kontoStatus = await konto.status();
      syncStatus = await sync.status();
    } catch (e) {
      syncFehler = e instanceof Error ? e.message : String(e);
    }
  }

  $effect(() => {
    datenVersion.wert;
    void statusLaden();
  });

  async function jetztAbgleichen() {
    syncLaeuft = true;
    syncFehler = null;
    letztesErgebnis = null;
    try {
      letztesErgebnis = await sync.jetzt();
      await statusLaden();
      datenVersion.bump();
    } catch (e) {
      syncFehler = e instanceof Error ? e.message : String(e);
    } finally {
      syncLaeuft = false;
    }
  }

  async function registrieren(e: Event) {
    e.preventDefault();
    if (syncLaeuft) return;
    syncLaeuft = true;
    syncFehler = null;
    try {
      await sync.registrieren(url.trim(), email.trim(), code.trim());
      code = '';
      registrierenOffen = false;
      await statusLaden();
    } catch (e2) {
      syncFehler = e2 instanceof Error ? e2.message : String(e2);
    } finally {
      syncLaeuft = false;
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

<section aria-labelledby="scan-titel">
  <h2 id="scan-titel">Ordner durchsuchen</h2>
  <p class="hinweis">
    Lotse sucht in den Unterordnern nach Erkennungsmarken und schlägt gefundene Projekte in der Hafeneinfahrt vor.
    Übernommen wird nichts von allein, und Dateiinhalte liest Lotse dabei nicht.
  </p>
  {#if echteDaten}
    <form class="zeile" onsubmit={scannen}>
      <input
        bind:value={wurzel}
        type="text"
        placeholder={navigator.platform.toLowerCase().startsWith('win') ? 'C:\\Users\\du\\Projekte' : '/home/du/Projekte'}
        aria-label="Wurzelordner"
      />
      <button type="submit" class="primaer" disabled={!wurzel.trim() || scanLaeuft}>
        {scanLaeuft ? 'Suche …' : 'Durchsuchen'}
      </button>
    </form>
    {#if scanErgebnis}
      <p class="ergebnis">{scanErgebnis} <a href="#/">Zum Hafen</a></p>
    {/if}
    {#if scanFehler}
      <p class="fehler">{scanFehler}</p>
    {/if}
  {:else}
    <p class="nur-desktop">Nur in der Desktop-App: der Browser hat keinen Zugriff auf deine Ordner.</p>
  {/if}
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
  {/if}
  {#if syncFehler}
    <p class="fehler">{syncFehler}</p>
  {/if}
</section>

{#if echteDaten}
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
    <button type="button" onclick={sperren}>Sperren</button>
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
  .hinweis {
    color: var(--text-gedaempft);
    max-width: 46rem;
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
  .ergebnis {
    margin-top: 0.7rem;
    color: var(--farbe-ruhig);
    font-size: 0.92rem;
  }
  .fehler {
    margin-top: 0.7rem;
    color: var(--farbe-ueberfaellig);
    font-size: 0.92rem;
  }
</style>
