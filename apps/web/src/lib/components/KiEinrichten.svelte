<script lang="ts">
  // Die KI-Einrichtung, angeboten statt versteckt.
  //
  // Bis 0.10 lag sie in Einstellungen → KI, mit einem Standardziel auf ein lokales Ollama,
  // das die wenigsten laufen haben. Wer sie nicht suchte, fand sie nie – und wer sie fand,
  // musste erst ein Modell abtippen. Deshalb fragt Lotse jetzt beim Start, einmal, deutlich.
  //
  // »Nicht mehr fragen« ist eine echte Antwort. Danach steht die Einrichtung nur noch in
  // den Einstellungen, und Lotse hält den Mund.
  import { untrack } from 'svelte';
  import { ki, KI_ZIELE, type KiStatus } from '../data/tauri';
  import { provider } from '../data/store';
  import { meldungen } from '../meldung.svelte';
  import type { TresorEintrag } from '../data/types';

  let { status, fertig }: { status: KiStatus; fertig: (neu?: KiStatus) => void } = $props();

  type Schritt = 'frage' | 'ollama' | 'schluessel' | 'modell';
  let schritt: Schritt = $state('frage');
  let laeuft = $state(false);
  let fehler = $state('');

  // Die Voreinstellung wird **einmal** gelesen, absichtlich: was jemand im Auswahlfeld
  // gewählt hat, darf nicht verschwinden, weil der Status neu hereinkommt. `untrack`
  // sagt das hin, statt es der Warnung zu überlassen.
  let url = $state(untrack(() => (status.ollama_da ? KI_ZIELE[0].url : KI_ZIELE[1].url)));
  let modelle: string[] = $state([]);
  let modell = $state('');
  let eintraege: TresorEintrag[] = $state([]);
  let eintragId = $state('');
  let feld = $state('schluessel');
  const lokal = $derived(url.startsWith('http://localhost') || url.startsWith('http://127.0.0.1'));

  async function nichtMehrFragen() {
    try {
      await ki.ablehnen(true);
      meldungen.zeigen('Gut. Die Einrichtung steht weiter unter Einstellungen → KI.');
    } catch (e) {
      meldungen.fehler(e);
    }
    fertig();
  }

  async function weiter() {
    fehler = '';
    if (lokal) {
      schritt = 'ollama';
      await modelleHolen();
      return;
    }
    schritt = 'schluessel';
    try {
      eintraege = await provider.listAllVaultEntries();
    } catch (e) {
      fehler = e instanceof Error ? e.message : String(e);
    }
  }

  async function modelleHolen() {
    fehler = '';
    laeuft = true;
    try {
      modelle = await ki.modelle(url);
      modell = modelle[0] ?? '';
      schritt = 'modell';
    } catch (e) {
      fehler = e instanceof Error ? e.message : String(e);
      // Kein Modell heißt nicht »Ende«: die Adresse kann stimmen und der Dienst nur
      // keine Liste anbieten. Dann wird der Name eingetippt.
      schritt = 'modell';
    } finally {
      laeuft = false;
    }
  }

  async function speichern() {
    if (!modell.trim()) {
      fehler = 'Ohne Modell geht es nicht.';
      return;
    }
    laeuft = true;
    fehler = '';
    try {
      await ki.zielSetzen(url.trim(), modell.trim(), lokal ? '' : eintragId, lokal ? '' : feld);
      const neu = await ki.status();
      meldungen.zeigen(`KI eingerichtet: ${modell.trim()}.`);
      fertig(neu);
    } catch (e) {
      fehler = e instanceof Error ? e.message : String(e);
    } finally {
      laeuft = false;
    }
  }
</script>

<div class="karte" role="region" aria-label="KI einrichten">
  {#if schritt === 'frage'}
    <h2>Soll Lotse mitdenken?</h2>
    <p>
      Ein Sprachmodell kann beim Einlesen aus einem Ordner ein beschriebenes Vorhaben machen:
      Titel in Worten statt Ordnername, ein Satz zum Ziel, Themen und die offenen Fäden, die
      schon dastehen. Und es fasst zusammen, was seit dem letzten Besuch passiert ist.
    </p>
    <p class="hinweis">
      Du siehst jedes Mal den vollständigen Text, bevor etwas das Gerät verlässt. Der Tresor
      ist für die KI unerreichbar. Ohne Einrichtung funktioniert Lotse vollständig weiter –
      es schreibt dann nur mit statt mitzudenken.
    </p>
    <div class="zeile">
      <select bind:value={url} aria-label="Ziel">
        {#each KI_ZIELE as z (z.url)}
          <option value={z.url}>{z.name}{z.url === KI_ZIELE[0].url && status.ollama_da ? ' – läuft' : ''}</option>
        {/each}
      </select>
      <button type="button" class="primaer" onclick={weiter}>Einrichten</button>
    </div>
    <div class="zeile schwach">
      <button type="button" onclick={() => fertig()}>Später</button>
      <button type="button" onclick={nichtMehrFragen}>Nicht mehr fragen</button>
    </div>
  {:else if schritt === 'ollama'}
    <h2>Ollama wird gefragt</h2>
    <p class="hinweis">{url}</p>
  {:else if schritt === 'schluessel'}
    <h2>Schlüssel</h2>
    <p class="hinweis">
      Der Schlüssel liegt im Tresor, nicht in einer Einstellung – Lotse merkt sich nur, wo.
      Lege ihn unter <a href="#/tresor">Tresor</a> ab, falls er noch nicht dort steht.
    </p>
    <label>
      <span>Eintrag</span>
      <select bind:value={eintragId}>
        <option value="">– wählen –</option>
        {#each eintraege as e (e.id)}
          <option value={e.id}>{e.titel}</option>
        {/each}
      </select>
    </label>
    <label>
      <span>Feld</span>
      <input bind:value={feld} type="text" placeholder="schluessel" />
    </label>
    <div class="zeile">
      <button type="button" onclick={() => (schritt = 'frage')}>Zurück</button>
      <button type="button" class="primaer" onclick={modelleHolen} disabled={!eintragId || laeuft}>
        {laeuft ? 'Frage nach …' : 'Modelle holen'}
      </button>
    </div>
  {:else}
    <h2>Modell</h2>
    {#if modelle.length}
      <label>
        <span>Verfügbar</span>
        <select bind:value={modell}>
          {#each modelle as m (m)}
            <option value={m}>{m}</option>
          {/each}
        </select>
      </label>
    {:else}
      <label>
        <span>Name des Modells</span>
        <input bind:value={modell} type="text" placeholder="z. B. llama3.2 oder gemini-2.0-flash" />
      </label>
    {/if}
    <div class="zeile">
      <button type="button" onclick={() => (schritt = 'frage')}>Zurück</button>
      <button type="button" class="primaer" onclick={speichern} disabled={laeuft}>Fertig</button>
    </div>
  {/if}

  {#if fehler}
    <p class="hinweis fehler">{fehler}</p>
  {/if}
</div>

<style>
  .karte {
    border: 1px solid var(--accent);
    border-left-width: 4px;
    border-radius: var(--radius, 10px);
    background: var(--surface);
    padding: 1rem 1.1rem 1.1rem;
    display: grid;
    gap: 0.6rem;
    margin-bottom: 1.4rem;
  }
  h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  p {
    margin: 0;
  }
  .zeile {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }
  .zeile.schwach {
    justify-content: flex-end;
    font-size: 0.9rem;
  }
  label {
    display: grid;
    gap: 0.25rem;
  }
  label span {
    font-size: 0.85rem;
    color: var(--muted, inherit);
  }
</style>
