<script lang="ts">
  import { konto, type Geheimnisse, type KontoStatus } from '../data/tauri';

  let { fertig }: { fertig: () => void } = $props();

  let status = $state<KontoStatus | null>(null);
  let modus = $state<'laden' | 'einrichten' | 'geheimnisse' | 'bestaetigen' | 'entsperren' | 'login' | 'wiederherstellen'>('laden');
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

  // Vergessenes Passwort: mit dem Wiederherstellungscode öffnen und dabei ein neues
  // Passwort setzen. Ohne diesen Weg wäre die App hier eine Sackgasse.
  async function wiederherstellen(ev: SubmitEvent) {
    ev.preventDefault();
    fehler = '';
    if (passwort.length < 12) return (fehler = 'Das neue Passwort braucht mindestens 12 Zeichen.');
    if (passwort !== passwort2) return (fehler = 'Die Passwörter stimmen nicht überein.');
    beschaeftigt = true;
    try {
      await konto.wiederherstellen(codeEingabe.trim(), passwort);
      codeEingabe = '';
      fertig();
    } catch (e) {
      fehler = String(e);
    } finally {
      beschaeftigt = false;
      passwort = passwort2 = '';
    }
  }

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

<!-- Die Tür ist der eine Ort, an dem das Marineblau des Logos die Fläche ist – auch
     im hellen Thema. Draußen ist Nacht, drinnen brennt Licht, und lange Texte liest
     hier niemand, also kostet der dunkle Grund nichts. -->
<div class="tuer">
  <svg class="wellen" viewBox="0 0 1440 400" preserveAspectRatio="none" aria-hidden="true">
    <g fill="none" stroke="#2e5f7a" stroke-width="2" opacity="0.4">
      <path d="M-40 190 C 240 150, 420 240, 720 190 S 1200 130, 1480 200" />
      <path d="M-40 248 C 240 208, 420 298, 720 248 S 1200 188, 1480 258" />
      <path d="M-40 306 C 240 266, 420 356, 720 306 S 1200 246, 1480 316" />
    </g>
  </svg>

  <div class="karte">
    <svg class="marke" viewBox="0 0 256 256" width="56" height="56" aria-hidden="true">
      <rect width="256" height="256" rx="56" fill="#14304a" />
      <g fill="none" stroke="#2e5f7a" stroke-width="9" opacity="0.8">
        <path d="M-10 200 C 40 180, 70 220, 120 200 S 200 180, 270 205" />
        <path d="M-10 222 C 40 202, 70 242, 120 222 S 200 202, 270 227" />
      </g>
      <path d="M80 56h32v112h64v32H80z" fill="#f2b544" />
      <circle cx="184" cy="72" r="20" fill="#e8e1d1" />
    </svg>
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
    <div class="geheimnis">
      <div class="g-kopf">
        <span class="g-name">Wiederherstellungscode</span>
        <span class="g-zweck">öffnet das Konto, wenn du das Passwort vergisst</span>
      </div>
      <code class="g-wert">{geheimnisse.wiederherstellungscode}</code>
    </div>
    <div class="geheimnis">
      <div class="g-kopf">
        <span class="g-name">Desktop-Schlüssel</span>
        <span class="g-zweck">nötig für Zugänge der Stufe „nur Desktop“</span>
      </div>
      <code class="g-wert">{geheimnisse.desktop_schluessel}</code>
    </div>
    <p class="warnband">
      In den Passwortmanager, nicht auf einen Zettel und nicht in eine Notiz-App. Ohne diesen Code ist ein vergessenes
      Passwort endgültig.
    </p>
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
    <button class="leise" type="button" onclick={() => { modus = 'wiederherstellen'; fehler = ''; }}>
      Passwort vergessen?
    </button>
  {:else if modus === 'wiederherstellen'}
    <h1>Konto wiederherstellen</h1>
    <p class="gedaempft">
      Gib den Wiederherstellungscode ein, den Lotse bei der Einrichtung einmal angezeigt hat, und setze ein neues
      Master-Passwort. Der Code bleibt danach derselbe.
    </p>
    <form onsubmit={wiederherstellen}>
      <label>Wiederherstellungscode <input type="text" bind:value={codeEingabe} autocomplete="off" spellcheck="false" placeholder="5YD6-AR19-…" required /></label>
      <label>Neues Master-Passwort <input type="password" bind:value={passwort} autocomplete="new-password" required /></label>
      <label>Noch einmal <input type="password" bind:value={passwort2} autocomplete="new-password" required /></label>
      <button type="submit" disabled={beschaeftigt}>{beschaeftigt ? 'Stelle wieder her …' : 'Wiederherstellen'}</button>
    </form>
    <button class="leise" type="button" onclick={() => { modus = 'entsperren'; fehler = ''; }}>Zurück</button>
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
</div>

<style>
  .tuer {
    position: fixed;
    inset: 0;
    background: linear-gradient(175deg, #14304a 0%, #0b1b2b 62%, #081521 100%);
    color: #e8e1d1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem 1.25rem;
    overflow: auto;
  }
  .wellen {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 40%;
    width: 100%;
    pointer-events: none;
  }

  .karte {
    position: relative;
    width: min(32rem, 100%);
    display: grid;
    gap: 0.9rem;
    justify-items: stretch;
  }
  .marke {
    justify-self: center;
    border-radius: 1rem;
    box-shadow: 0 10px 34px rgba(0, 0, 0, 0.42);
    margin-bottom: 0.5rem;
  }
  h1 {
    font-family: ui-serif, Georgia, 'Iowan Old Style', 'Times New Roman', serif;
    font-size: 1.7rem;
    font-weight: 600;
    margin: 0;
    text-align: center;
  }
  form {
    display: grid;
    gap: 0.8rem;
  }
  label {
    display: grid;
    gap: 0.3rem;
    font-size: 0.78rem;
    letter-spacing: 0.03em;
    color: #8fa8bd;
  }
  label.zeile {
    grid-template-columns: auto 1fr;
    align-items: center;
    font-size: 0.85rem;
    color: #e8e1d1;
  }
  input[type='text'],
  input[type='password'],
  input[type='url'],
  input[type='email'] {
    font: inherit;
    font-size: 0.95rem;
    padding: 0.7rem 0.85rem;
    border: 1px solid #2c4b68;
    border-radius: 0.6rem;
    background: rgba(255, 255, 255, 0.045);
    color: #e8e1d1;
  }
  input:focus {
    outline: 2px solid #f2b544;
    outline-offset: 1px;
  }

  /* Der Weg nach vorn trägt Bernstein, alles andere bleibt still. */
  button {
    font: inherit;
    font-weight: 600;
    padding: 0.7rem 1rem;
    border-radius: 0.6rem;
    border: 1px solid #f2b544;
    background: #f2b544;
    color: #14304a;
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    filter: brightness(1.06);
  }
  /* Gesperrt wird neutral, nicht blass: Bernstein bei halber Deckkraft sagt weder
     „aus“ noch „an“. */
  button:disabled {
    background: transparent;
    border-color: #2c4b68;
    color: #6d869b;
    font-weight: 400;
    cursor: default;
  }
  button.leise {
    background: transparent;
    color: #8fa8bd;
    border-color: transparent;
    justify-self: center;
    font-weight: 400;
    font-size: 0.83rem;
    padding: 0.3rem 0.5rem;
  }
  button.leise:hover {
    color: #e8e1d1;
    filter: none;
  }

  .geheimnis {
    border: 1px solid #2c4b68;
    border-radius: 0.7rem;
    background: rgba(255, 255, 255, 0.04);
    padding: 0.8rem 0.9rem;
  }
  .g-kopf {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    flex-wrap: wrap;
    margin-bottom: 0.55rem;
  }
  .g-name {
    font-weight: 600;
    font-size: 0.85rem;
  }
  .g-zweck {
    font-size: 0.72rem;
    color: #8fa8bd;
  }
  /* Der Wert selbst bekommt das Bernstein: er ist das Einzige auf dieser Seite, das
     man wirklich mitnehmen muss. */
  .g-wert {
    display: block;
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 1rem;
    letter-spacing: 0.06em;
    color: #f2b544;
    background: rgba(0, 0, 0, 0.22);
    border-radius: 0.45rem;
    padding: 0.65rem 0.8rem;
    user-select: all;
    overflow-wrap: anywhere;
  }
  .warnband {
    margin: 0;
    padding: 0.7rem 0.85rem;
    border: 1px solid rgba(242, 181, 68, 0.4);
    border-left: 3px solid #f2b544;
    border-radius: 0.6rem;
    background: rgba(242, 181, 68, 0.07);
    font-size: 0.8rem;
    line-height: 1.55;
    color: #e3d9c6;
  }

  code {
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    letter-spacing: 0.04em;
    user-select: all;
  }
  .gedaempft {
    color: #a9bccb;
    margin: 0;
    font-size: 0.85rem;
    line-height: 1.6;
    text-align: center;
  }
  .fehler {
    color: #ffb3a7;
    margin: 0;
    font-size: 0.85rem;
    text-align: center;
  }
</style>
