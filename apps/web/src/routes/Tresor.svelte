<script lang="ts">
  // Alle Zugänge, projektübergreifend. Werte bleiben verdeckt, bis sie einzeln
  // angefordert werden – der Provider hält nie Klartext.
  import NeuerTresorEintrag from '../lib/components/NeuerTresorEintrag.svelte';
  import TresorListe from '../lib/components/TresorListe.svelte';
  import { echteDaten, provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';

  let neuOffen = $state(false);
  let filter = $state('');

  async function laden() {
    const [eintraege, projekte] = await Promise.all([provider.listAllVaultEntries(), provider.listProjects()]);
    return { eintraege, projekte };
  }

  let datenPromise = $derived.by(() => {
    datenVersion.wert;
    return laden();
  });
</script>

<svelte:head><title>Tresor · Lotse</title></svelte:head>

{#await datenPromise then { projekte }}
  <NeuerTresorEintrag bind:offen={neuOffen} {projekte} />
{/await}

<div class="seiten-kopf">
  <h1>Tresor</h1>
  <button type="button" class="primaer" onclick={() => (neuOffen = true)}>Neuer Zugang</button>
</div>

<p class="hinweis einleitung">
  Zugangsdaten am Projekt, Ende-zu-Ende verschlüsselt. Einträge der Stufe <strong>nur Desktop</strong> lassen sich nur auf
  diesem Rechner öffnen, weil dafür zusätzlich der Desktop-Schlüssel aus dem Schlüsselbund nötig ist.
</p>

{#if !echteDaten}
  <p class="hinweis beispiel">Beispieldaten im Browser. In der Desktop-App stehen hier deine echten Zugänge.</p>
{/if}

{#await datenPromise}
  <p class="hinweis">Lade Tresor…</p>
{:then { eintraege, projekte }}
  {#if eintraege.length > 0}
    <input class="filter" bind:value={filter} type="search" placeholder="Nach Titel filtern …" aria-label="Tresor filtern" />
  {/if}
  {@const gefiltert = filter.trim()
    ? eintraege.filter((e) => e.titel.toLowerCase().includes(filter.trim().toLowerCase()))
    : eintraege}
  {#if eintraege.length === 0}
    <p class="hinweis">
      Noch keine Zugänge. Was projektgebunden ist und sonst in einer Textdatei landen würde, gehört hierher – Web-Logins
      bleiben im Passwortmanager.
    </p>
  {:else if gefiltert.length === 0}
    <p class="hinweis">Kein Eintrag passt zum Filter.</p>
  {:else}
    <TresorListe eintraege={gefiltert} {projekte} zeigeProjekte />
  {/if}
{:catch fehler}
  <p class="fehler">Tresor konnte nicht geladen werden: {fehler.message}</p>
{/await}

<style>
  .seiten-kopf {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    margin-bottom: 0.75rem;
  }
  .seiten-kopf h1 {
    margin: 0;
    font-size: 1.5rem;
  }
  .primaer {
    border: 1px solid var(--akzent);
    background: var(--karten-hintergrund);
    color: var(--akzent);
    font-weight: 600;
    border-radius: 0.4rem;
    padding: 0.4rem 0.85rem;
  }
  .einleitung {
    max-width: 46rem;
    margin: 0 0 1rem;
  }
  .beispiel {
    border: 1px dashed var(--rahmen);
    border-radius: 0.5rem;
    padding: 0.6rem 0.8rem;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }
  .filter {
    width: min(22rem, 100%);
    margin-bottom: 1rem;
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
</style>
