<script lang="ts">
  // Projekt von Hand anlegen. Die Vorlage setzt nur Defaults (Erwartungsintervall,
  // Erkennungsmarken); das Datenmodell ist für alle Projektarten dasselbe.
  import { provider } from '../data/store';
  import { datenVersion } from '../data/version.svelte';
  import { meldungen } from '../meldung.svelte';
  import { VORLAGEN_LABEL } from '../format';
  import { navigiereZu } from '../router.svelte';
  import type { VorlagenId } from '../data/types';

  let { offen = $bindable(false) }: { offen?: boolean } = $props();

  const VORLAGEN = Object.keys(VORLAGEN_LABEL) as VorlagenId[];

  let titel = $state('');
  let vorlage: VorlagenId = $state('generisch');
  let kurs = $state('');
  let wirdGespeichert = $state(false);
  let fehler: string | null = $state(null);
  let titelEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (offen) {
      titel = '';
      vorlage = 'generisch';
      kurs = '';
      fehler = null;
      titelEl?.focus();
    }
  });

  async function anlegen(e: Event) {
    e.preventDefault();
    if (!titel.trim() || wirdGespeichert) return;
    wirdGespeichert = true;
    fehler = null;
    try {
      const projekt = await provider.createProject(titel.trim(), vorlage, kurs.trim() || undefined);
      datenVersion.bump();
      offen = false;
      meldungen.zeigen(`„${projekt.titel}“ angelegt.`);
      navigiereZu(`#/projekt/${projekt.id}`);
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
    <form class="dialog" onsubmit={anlegen} aria-label="Neues Projekt">
      <h2>Neues Projekt</h2>

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

      {#if fehler}
        <p class="fehler">{fehler}</p>
      {/if}

      <div class="aktionen">
        <button type="button" onclick={() => (offen = false)}>Abbrechen</button>
        <button type="submit" class="primaer" disabled={!titel.trim() || wirdGespeichert}>
          {wirdGespeichert ? 'Lege an …' : 'Anlegen'}
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
    padding-top: 10vh;
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
    width: min(30rem, 92vw);
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
  label span {
    color: var(--text-gedaempft);
  }
  label em {
    font-style: normal;
    opacity: 0.7;
  }
  input,
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
