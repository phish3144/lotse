<script lang="ts">
  import { konto, type Geheimnisse, type KontoStatus } from '../data/tauri';

  let { fertig }: { fertig: () => void } = $props();

  let status = $state<KontoStatus | null>(null);
  let modus = $state<'laden' | 'einrichten' | 'geheimnisse' | 'bestaetigen' | 'entsperren' | 'login'>('laden');
  let fehler = $state('');
  let beschaeftigt = $state(false);

  let passwort = $state('');
  let passwort2 = $state('');
  let geraet = $state('Dieser Rechner');
  let desktopSchluessel = $state('');
  let geheimnisse = $state<Geheimnisse | null>(null);
  let codeEingabe = $state('');
  let gesichert = $state(false);

  let syncUrl = $state('');
  let email = $state('');

  $effect(() => {
    konto
      .status()
      .then((s) => {
        status = s;
        modus = s.entsperrt ? 'laden' : s.eingerichtet ? 'entsperren' : 'einrichten';
        if (s.entsperrt) fertig();
      })
      .catch((e) => (fehler = String(e)));
  });

  async function einrichten(ev: SubmitEvent) {
    ev.preventDefault();
    fehler = '';
    if (passwort.length < 12) return (fehler = 'Mindestens 12 Zeichen.');
    if (passwort !== passwort2) return (fehler = 'Die Passwörter stimmen nicht überein.');
    beschaeftigt = true;
    try {
      geheimnisse = await konto.einrichten(passwort, geraet.trim() || 'Dieser Rechner');
      modus = 'geheimnisse';
    } catch (e) {
      fehler = String(e);
    } finally {
      beschaeftigt = false;
      passwort = passwort2 = '';
    }
  }

  async function bestaetigen(ev: SubmitEvent) {
    ev.preventDefault();
    fehler = '';
    beschaeftigt = true;
    try {
      if (await konto.codePruefen(codeEingabe)) {
        geheimnisse = null;
        codeEingabe = '';
        fertig();
      } else {
        fehler = 'Der Code stimmt nicht. Bitte genau so eingeben, wie er angezeigt wurde.';
      }
    } finally {
      beschaeftigt = false;
    }
  }

  async function entsperren(ev: SubmitEvent) {
    ev.preventDefault();
    fehler = '';
    beschaeftigt = true;
    try {
      await konto.entsperren(passwort, desktopSchluessel.trim() || undefined);
      fertig();
    } catch (e) {
      fehler = String(e);
    } finally {
      beschaeftigt = false;
      passwort = '';
    }
  }

  async function login(ev: SubmitEvent) {
    ev.preventDefault();
    fehler = '';
    beschaeftigt = true;
    try {
      await konto.syncLogin(syncUrl.trim(), email.trim(), passwort, geraet.trim() || 'Dieser Rechner');
      fertig();
    } catch (e) {
      fehler = String(e);
    } finally {
      beschaeftigt = false;
      passwort = '';
    }
  }
</script>

<div class="karte">
  {#if modus === 'laden'}
    <p class="gedaempft">Lotse startet …</p>
  {:else if modus === 'einrichten'}
    <h1>Willkommen an Bord.</h1>
    <p class="gedaempft">Ein Master-Passwort schützt alles auf diesem Rechner und beim Abgleich. Es lässt sich nicht zurücksetzen, nur mit dem Wiederherstellungscode ersetzen, den du gleich bekommst.</p>
    <form onsubmit={einrichten}>
      <label>Master-Passwort <input type="password" bind:value={passwort} autocomplete="new-password" required /></label>
      <label>Noch einmal <input type="password" bind:value={passwort2} autocomplete="new-password" required /></label>
      <label>Name dieses Rechners <input type="text" bind:value={geraet} /></label>
      <button type="submit" disabled={beschaeftigt}>{beschaeftigt ? 'Leite Schlüssel ab …' : 'Konto einrichten'}</button>
    </form>
    <button class="leise" type="button" onclick={() => (modus = 'login')}>Ich habe schon ein Konto auf einem anderen Rechner</button>
  {:else if modus === 'geheimnisse' && geheimnisse}
    <h1>Zwei Dinge, die du jetzt sicher ablegst.</h1>
    <p class="gedaempft">Beides wird genau einmal angezeigt. Leg es in deinen Passwortmanager.</p>
    <dl>
      <dt>Wiederherstellungscode</dt>
      <dd><code>{geheimnisse.wiederherstellungscode}</code></dd>
      <dt>Desktop-Schlüssel</dt>
      <dd><code>{geheimnisse.desktop_schluessel}</code></dd>
    </dl>
    <p class="gedaempft">
      {#if geheimnisse.schluesselbund}
        Der Desktop-Schlüssel liegt zusätzlich im Schlüsselbund dieses Rechners. Auf einem weiteren Rechner gibst du ihn einmal ein.
      {:else}
        Kein Schlüsselbund verfügbar. Du wirst beim Entsperren nach dem Desktop-Schlüssel gefragt, sonst bleiben Einträge der Stufe »nur Desktop« unlesbar.
      {/if}
    </p>
    <label class="zeile"><input type="checkbox" bind:checked={gesichert} /> Ich habe beides gespeichert.</label>
    <button type="button" disabled={!gesichert} onclick={() => (modus = 'bestaetigen')}>Weiter</button>
  {:else if modus === 'bestaetigen'}
    <h1>Zur Sicherheit: den Wiederherstellungscode einmal eingeben.</h1>
    <form onsubmit={bestaetigen}>
      <label>Wiederherstellungscode <input type="text" bind:value={codeEingabe} autocomplete="off" spellcheck="false" required /></label>
      <button type="submit" disabled={beschaeftigt}>Prüfen und loslegen</button>
    </form>
  {:else if modus === 'entsperren'}
    <h1>Lotse entsperren</h1>
    {#if status?.geraet}<p class="gedaempft">Gerät »{status.geraet}«</p>{/if}
    <form onsubmit={entsperren}>
      <label>Master-Passwort <input type="password" bind:value={passwort} autocomplete="current-password" required /></label>
      <details>
        <summary>Desktop-Schlüssel eingeben (nur nötig, wenn kein Schlüsselbund vorhanden ist)</summary>
        <label>Desktop-Schlüssel <input type="text" bind:value={desktopSchluessel} autocomplete="off" spellcheck="false" /></label>
      </details>
      <button type="submit" disabled={beschaeftigt}>{beschaeftigt ? 'Entsperre …' : 'Entsperren'}</button>
    </form>
  {:else if modus === 'login'}
    <h1>Diesen Rechner anmelden</h1>
    <p class="gedaempft">Der Sync-Dienst liefert deinen Konto-Header, das Master-Passwort entsperrt ihn hier. Der Dienst sieht das Passwort nie.</p>
    <form onsubmit={login}>
      <label>Adresse des Sync-Dienstes <input type="url" bind:value={syncUrl} placeholder="https://api.example.invalid" required /></label>
      <label>E-Mail <input type="email" bind:value={email} required /></label>
      <label>Master-Passwort <input type="password" bind:value={passwort} autocomplete="current-password" required /></label>
      <label>Name dieses Rechners <input type="text" bind:value={geraet} /></label>
      <button type="submit" disabled={beschaeftigt}>{beschaeftigt ? 'Melde an …' : 'Anmelden und herunterladen'}</button>
    </form>
    <button class="leise" type="button" onclick={() => (modus = 'einrichten')}>Zurück</button>
  {/if}
  {#if fehler}<p class="fehler" role="alert">{fehler}</p>{/if}
</div>

<style>
  .karte {
    max-width: 30rem;
    margin: 4rem auto;
    padding: 1.5rem;
    border: 1px solid var(--rahmen);
    border-radius: 10px;
    display: grid;
    gap: 0.9rem;
  }
  h1 {
    font-size: 1.4rem;
    margin: 0;
  }
  form {
    display: grid;
    gap: 0.8rem;
  }
  label {
    display: grid;
    gap: 0.3rem;
    font-size: 0.9rem;
  }
  label.zeile {
    grid-template-columns: auto 1fr;
    align-items: center;
  }
  input[type='text'],
  input[type='password'],
  input[type='url'],
  input[type='email'] {
    font: inherit;
    padding: 0.5rem 0.6rem;
    border: 1px solid var(--rahmen);
    border-radius: 6px;
    background: transparent;
    color: inherit;
  }
  button {
    font: inherit;
    padding: 0.6rem 1rem;
    border-radius: 999px;
    border: 1px solid var(--akzent);
    background: var(--akzent);
    color: var(--bg, #fff);
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.6;
    cursor: default;
  }
  button.leise {
    background: transparent;
    color: var(--text-gedaempft);
    border-color: transparent;
    justify-self: start;
    padding-left: 0;
  }
  dl {
    display: grid;
    gap: 0.3rem;
    margin: 0;
  }
  dt {
    font-size: 0.8rem;
    color: var(--text-gedaempft);
  }
  dd {
    margin: 0 0 0.5rem;
  }
  code {
    font-size: 1.05rem;
    letter-spacing: 0.04em;
    user-select: all;
  }
  .gedaempft {
    color: var(--text-gedaempft);
    margin: 0;
  }
  .fehler {
    color: #c0392b;
    margin: 0;
  }
</style>
