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

{#await datenPromise}
  <div class="seiten-kopf"><h1 class="serife">Tresor</h1></div>
  <p class="hinweis">Lade Tresor…</p>
{:then { eintraege, projekte }}
  {@const gefiltert = filter.trim()
    ? eintraege.filter((e) => e.titel.toLowerCase().includes(filter.trim().toLowerCase()))
    : eintraege}
  {@const nurDesktop = eintraege.filter((e) => e.stufe === 'nur_desktop').length}

  <div class="seiten-kopf">
    <h1 class="serife">Tresor</h1>
    <span class="kopf-meta">
      {eintraege.length}
      {eintraege.length === 1 ? 'Zugang' : 'Zugänge'}
      {#if nurDesktop > 0}· {nurDesktop} nur auf diesem Rechner{/if}
    </span>
    <div class="kopf-aktionen">
      {#if eintraege.length > 0}
        <input
          class="filter"
          bind:value={filter}
          type="search"
          placeholder="Nach Titel filtern …"
          aria-label="Tresor filtern"
        />
      {/if}
      <button type="button" class="primaer" onclick={() => (neuOffen = true)}>Neuer Zugang</button>
    </div>
  </div>

  <div class="tresor-flaeche">
    <main class="tresor-haupt">
      {#if !echteDaten}
        <p class="hinweis beispiel">Beispieldaten im Browser. In der Desktop-App stehen hier deine echten Zugänge.</p>
      {/if}
      {#if eintraege.length === 0}
        <p class="hinweis">
          Noch keine Zugänge. Was projektgebunden ist und sonst in einer Textdatei landen würde, gehört hierher –
          Web-Logins bleiben im Passwortmanager.
        </p>
      {:else if gefiltert.length === 0}
        <p class="hinweis">Kein Eintrag passt zum Filter.</p>
      {:else}
        <TresorListe eintraege={gefiltert} {projekte} zeigeProjekte />
      {/if}
    </main>

    <aside class="tresor-neben">
      <section class="karte">
        <h2>Was hierher gehört</h2>
        <p>
          Was an einem Vorhaben hängt und sonst in einer Textdatei neben dem Projekt landen würde: der Router im
          Gartenhaus, der Admin-Login der eigenen App, die Kontonummer des Vereins.
        </p>
        <p><strong>Nicht hierher:</strong> die Web-Logins des Alltags. Dafür gibt es Passwortmanager.</p>
      </section>

      {#if nurDesktop > 0}
        <section class="karte">
          <h2>Nur Desktop <span class="zahl">{nurDesktop}</span></h2>
          <p>
            Diese brauchen zusätzlich den Desktop-Schlüssel aus dem Schlüsselbund. Sie gehen beim Abgleich mit, aber ein
            fremder Rechner kann sie nicht öffnen – auch dann nicht, wenn jemand dein Master-Passwort kennt.
          </p>
        </section>
      {/if}

      <section class="karte">
        <h2>Zwischenablage</h2>
        <p>Kopierte Werte räumt Lotse nach 30 Sekunden weg – aber nur, wenn seither nichts anderes kopiert wurde.</p>
      </section>
    </aside>
  </div>
{:catch fehler}
  <p class="fehler">Tresor konnte nicht geladen werden: {fehler.message}</p>
{/await}

<style>
  .seiten-kopf {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    flex-wrap: wrap;
    margin-bottom: 1.25rem;
  }
  .seiten-kopf h1 {
    margin: 0;
    font-size: 1.7rem;
    font-weight: 600;
  }
  .kopf-meta {
    font-size: 0.8rem;
    color: var(--text-gedaempft);
  }
  .kopf-aktionen {
    margin-left: auto;
    display: flex;
    gap: 0.5rem;
    align-self: center;
  }
  .filter {
    width: min(15rem, 100%);
  }
  .kopf-aktionen .primaer {
    white-space: nowrap;
  }

  /* Der Tresor ist eine Liste mit Vorbehalten: was hierher gehört und was nicht, was
     „nur Desktop“ bedeutet, wie lange Kopiertes liegen bleibt. Das gehört daneben,
     nicht als Vorwort über die Einträge. */
  .tresor-flaeche {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 22rem;
    gap: 1.75rem;
    align-items: start;
  }
  .tresor-haupt {
    min-width: 0;
  }
  .tresor-neben {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    position: sticky;
    top: 1rem;
  }
  @media (max-width: 64rem) {
    .tresor-flaeche {
      grid-template-columns: minmax(0, 1fr);
    }
    .tresor-neben {
      position: static;
    }
  }

  .karte {
    border: 1px solid var(--rahmen);
    border-radius: 0.7rem;
    background: var(--karten-hintergrund);
    padding: 0.75rem 0.85rem;
    box-shadow: var(--schatten);
  }
  .karte h2 {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-gedaempft);
    margin: 0 0 0.5rem;
  }
  .zahl {
    font-size: 0.7rem;
    background: var(--flaeche-still);
    border-radius: 999px;
    padding: 0.05rem 0.4rem;
    letter-spacing: 0;
  }
  .karte p {
    margin: 0 0 0.5rem;
    font-size: 0.8rem;
    line-height: 1.5;
    color: var(--text-gedaempft);
  }
  .karte p:last-child {
    margin-bottom: 0;
  }
  .karte strong {
    color: var(--fg);
  }

  .beispiel {
    border: 1px dashed var(--rahmen);
    border-radius: 0.5rem;
    padding: 0.6rem 0.8rem;
    margin: 0 0 1rem;
    font-size: 0.9rem;
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
</style>
