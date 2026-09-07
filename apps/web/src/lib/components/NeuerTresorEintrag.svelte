<script lang="ts">
  // Neuer Tresor-Eintrag. Die Klartextwerte gehen genau einmal über die Provider-Grenze
  // und werden hier nach dem Speichern sofort verworfen.
  import { provider } from '../data/store';
  import { datenVersion } from '../data/version.svelte';
  import { meldungen } from '../meldung.svelte';
  import { STUFE_LABEL } from '../format';
  import type { Id, Projekt, TresorStufe } from '../data/types';

  let {
    offen = $bindable(false),
    projekte = [],
    vorausgewaehlt,
  }: { offen?: boolean; projekte?: Projekt[]; vorausgewaehlt?: Id } = $props();

  interface FeldEingabe {
    name: string;
    wert: string;
  }

  let titel = $state('');
  let stufe: TresorStufe = $state('ueberall');
  let projektId: Id | '' = $state('');
  let felder: FeldEingabe[] = $state([{ name: 'Passwort', wert: '' }]);
  let wirdGespeichert = $state(false);
  let fehler: string | null = $state(null);
  let titelEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (offen) {
      titel = '';
      stufe = 'ueberall';
      projektId = vorausgewaehlt ?? '';
      felder = [{ name: 'Passwort', wert: '' }];
      fehler = null;
      titelEl?.focus();
    }
  });

  function feldHinzufuegen() {
    felder = [...felder, { name: '', wert: '' }];
  }

  function feldEntfernen(index: number) {
    felder = felder.filter((_, i) => i !== index);
  }

  async function speichern(e: Event) {
    e.preventDefault();
    const gefuellt = felder.filter((f) => f.name.trim() && f.wert);
    if (!titel.trim() || gefuellt.length === 0 || wirdGespeichert) return;
    wirdGespeichert = true;
    fehler = null;
    try {
      await provider.addVaultEntry(
        titel.trim(),
        projektId ? [projektId] : [],
        stufe,
        gefuellt.map((f) => ({ name: f.name.trim(), wert: f.wert })),
      );
      // Werte nicht länger als nötig im Speicher der Oberfläche halten.
      felder = [{ name: 'Passwort', wert: '' }];
      datenVersion.bump();
      meldungen.zeigen('Zugang gespeichert.');
      offen = false;
    } catch (e2) {
      fehler = e2 instanceof Error ? e2.message : String(e2);
    } finally {
      wirdGespeichert = false;
    }
  }

  function aufTaste(e: KeyboardEvent) {
    if (e.key === 'Escape' && offen) offen = false;
  }
</script>

<svelte:window onkeydown={aufTaste} />

{#if offen}
  <div class="ueberlagerung">
    <button type="button" class="rueckwand" aria-label="Dialog schließen" onclick={() => (offen = false)}></button>
    <form class="dialog" onsubmit={speichern} aria-label="Neuer Zugang">
      <h2>Neuer Zugang</h2>

      <label>
        <span>Titel</span>
        <input bind:this={titelEl} bind:value={titel} type="text" placeholder="Fritzbox Gartenhaus" required />
      </label>

      <label>
        <span>Projekt</span>
        <select bind:value={projektId}>
          <option value="">Ohne Projekt</option>
          {#each projekte as p (p.id)}
            <option value={p.id}>{p.titel}</option>
          {/each}
        </select>
      </label>

      <fieldset class="stufe">
        <legend>Stufe</legend>
        <label class="radio">
          <input type="radio" bind:group={stufe} value="ueberall" />
          <span><strong>{STUFE_LABEL.ueberall}</strong> – auf allen deinen Geräten lesbar</span>
        </label>
        <label class="radio">
          <input type="radio" bind:group={stufe} value="nur_desktop" />
          <span><strong>{STUFE_LABEL.nur_desktop}</strong> – braucht zusätzlich den Desktop-Schlüssel</span>
        </label>
      </fieldset>

      <div class="felder">
        <span class="beschriftung">Felder</span>
        {#each felder as feld, i (i)}
          <div class="feld-zeile">
            <input bind:value={feld.name} type="text" placeholder="Feldname" aria-label="Feldname" />
            <input bind:value={feld.wert} type="password" placeholder="Wert" aria-label="Wert" autocomplete="off" />
            {#if felder.length > 1}
              <button type="button" onclick={() => feldEntfernen(i)} aria-label="Feld entfernen">×</button>
            {/if}
          </div>
        {/each}
        <button type="button" class="feld-plus" onclick={feldHinzufuegen}>Feld hinzufügen</button>
      </div>

      {#if fehler}
        <p class="fehler">{fehler}</p>
      {/if}

      <div class="aktionen">
        <button type="button" onclick={() => (offen = false)}>Abbrechen</button>
        <button type="submit" class="primaer" disabled={wirdGespeichert}>
          {wirdGespeichert ? 'Speichere …' : 'Speichern'}
        </button>
      </div>
    </form>
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
    overflow-y: auto;
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
    width: min(32rem, 92vw);
    margin-bottom: 3rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }
  h2 {
    margin: 0;
    font-size: 1.15rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.9rem;
  }
  label > span {
    color: var(--text-gedaempft);
  }
  input,
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
  fieldset.stufe {
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    padding: 0.6rem 0.8rem 0.7rem;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  legend {
    font-size: 0.9rem;
    color: var(--text-gedaempft);
    padding: 0 0.3rem;
  }
  .radio {
    flex-direction: row;
    align-items: baseline;
    gap: 0.5rem;
    font-size: 0.88rem;
  }
  .radio input {
    width: auto;
  }
  .felder {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .beschriftung {
    font-size: 0.9rem;
    color: var(--text-gedaempft);
  }
  .feld-zeile {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  .feld-zeile input:first-child {
    flex: 0 0 9rem;
  }
  .feld-plus {
    align-self: flex-start;
  }
  button {
    border: 1px solid var(--rahmen);
    background: transparent;
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.3rem 0.7rem;
    font-size: 0.85rem;
  }
  button:hover:not(:disabled) {
    border-color: var(--akzent);
  }
  .aktionen {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .aktionen button {
    padding: 0.45rem 0.9rem;
  }
  .aktionen .primaer {
    border-color: var(--akzent);
    color: var(--akzent);
    font-weight: 600;
  }
  button:disabled {
    opacity: 0.5;
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
    font-size: 0.85rem;
    margin: 0;
  }
</style>
