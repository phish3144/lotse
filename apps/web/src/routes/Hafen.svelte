<script lang="ts">
  import { auffaelligkeit, type Auffaelligkeit } from '../lib/brief';
  import ProjektKarte from '../lib/components/ProjektKarte.svelte';
  import { provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';
  import { VORLAGEN_LABEL } from '../lib/data/mock';
  import type { Kandidat, Notiz, Projekt } from '../lib/data/types';

  interface Eintrag {
    projekt: Projekt;
    letzteNotiz?: Notiz;
    grad: Auffaelligkeit;
  }

  const GRAD_RANG: Record<Auffaelligkeit, number> = { ueberfaellig: 0, auffaellig: 1, ruhig: 2 };

  async function laden() {
    const [projekte, kandidaten] = await Promise.all([provider.listProjects(), provider.listCandidates()]);
    const eintraege: Eintrag[] = await Promise.all(
      projekte.map(async (projekt) => {
        const notizen = await provider.listNotes(projekt.id);
        return { projekt, letzteNotiz: notizen[0], grad: auffaelligkeit(projekt, notizen) };
      }),
    );
    return { eintraege, kandidaten };
  }

  let datenPromise = $derived.by(() => {
    datenVersion.wert;
    return laden();
  });

  let bestaetigtWirdGerade: string | undefined = $state(undefined);

  async function bestaetigen(kandidat: Kandidat) {
    bestaetigtWirdGerade = kandidat.id;
    try {
      await provider.confirmCandidate(kandidat.id);
      datenVersion.bump();
    } finally {
      bestaetigtWirdGerade = undefined;
    }
  }

  function sortiertNachAuffaelligkeit(liste: Eintrag[]): Eintrag[] {
    return [...liste].sort(
      (a, b) => GRAD_RANG[a.grad] - GRAD_RANG[b.grad] || b.projekt.zuletzt_beruehrt.localeCompare(a.projekt.zuletzt_beruehrt),
    );
  }
</script>

<svelte:head><title>Hafen · Lotse</title></svelte:head>

{#await datenPromise}
  <p class="hinweis">Lade Hafen…</p>
{:then { eintraege, kandidaten }}
  {@const heuteWichtig = sortiertNachAuffaelligkeit(
    eintraege.filter((e) => e.grad !== 'ruhig' && e.projekt.status !== 'abgeschlossen' && e.projekt.status !== 'eingemottet'),
  ).slice(0, 3)}
  {@const aufSee = sortiertNachAuffaelligkeit(eintraege.filter((e) => e.projekt.status === 'aktiv'))}
  {@const vorAnker = sortiertNachAuffaelligkeit(eintraege.filter((e) => e.projekt.status === 'pausiert' || e.projekt.status === 'wartet'))}
  {@const ideen = sortiertNachAuffaelligkeit(eintraege.filter((e) => e.projekt.status === 'idee'))}

  {#if heuteWichtig.length > 0}
    <section aria-labelledby="heute-wichtig-titel">
      <h2 id="heute-wichtig-titel">Heute wichtig</h2>
      <div class="karten-raster">
        {#each heuteWichtig as e (e.projekt.id)}
          <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
        {/each}
      </div>
    </section>
  {/if}

  {#if kandidaten.length > 0}
    <section aria-labelledby="hafeneinfahrt-titel">
      <h2 id="hafeneinfahrt-titel">Hafeneinfahrt</h2>
      <p class="hinweis">Erkannte Projektkandidaten, noch nicht bestätigt.</p>
      <ul class="kandidaten-liste">
        {#each kandidaten as kandidat (kandidat.id)}
          <li class="kandidat">
            <div>
              <strong>{kandidat.titel_vorschlag}</strong>
              <div class="kandidat-details">
                {kandidat.pfad} · Vorlage: {VORLAGEN_LABEL[kandidat.vorlage_vorschlag]} · Marke: {kandidat.erkennungsmarke}
              </div>
            </div>
            <button type="button" onclick={() => bestaetigen(kandidat)} disabled={bestaetigtWirdGerade === kandidat.id}>
              {bestaetigtWirdGerade === kandidat.id ? 'Bestätige …' : 'Bestätigen'}
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <section aria-labelledby="auf-see-titel">
    <h2 id="auf-see-titel">Auf See</h2>
    {#if aufSee.length === 0}
      <p class="hinweis">Keine aktiven Projekte.</p>
    {:else}
      <div class="karten-raster">
        {#each aufSee as e (e.projekt.id)}
          <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
        {/each}
      </div>
    {/if}
  </section>

  {#if vorAnker.length > 0}
    <details class="gruppe">
      <summary><h2>Vor Anker ({vorAnker.length})</h2></summary>
      <div class="karten-raster">
        {#each vorAnker as e (e.projekt.id)}
          <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
        {/each}
      </div>
    </details>
  {/if}

  {#if ideen.length > 0}
    <section aria-labelledby="ideen-titel">
      <h2 id="ideen-titel">Ideen</h2>
      <div class="karten-raster">
        {#each ideen as e (e.projekt.id)}
          <ProjektKarte projekt={e.projekt} letzteNotiz={e.letzteNotiz} auffaelligkeit={e.grad} />
        {/each}
      </div>
    </section>
  {/if}
{:catch fehler}
  <p class="hinweis fehler">Hafen konnte nicht geladen werden: {fehler.message}</p>
{/await}

<style>
  section,
  .gruppe {
    margin-bottom: 2rem;
  }
  h2 {
    font-size: 1.05rem;
    margin: 0 0 0.75rem;
  }
  .gruppe summary {
    cursor: pointer;
    list-style: none;
  }
  .gruppe summary::-webkit-details-marker {
    display: none;
  }
  .gruppe summary h2 {
    display: inline-block;
    margin: 0 0 0.75rem;
  }
  .gruppe summary::before {
    content: '▸ ';
    color: var(--text-gedaempft);
  }
  .gruppe[open] summary::before {
    content: '▾ ';
  }
  .karten-raster {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
    gap: 0.75rem;
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
  .kandidaten-liste {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .kandidat {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border: 1px dashed var(--rahmen);
    border-radius: 0.6rem;
    background: var(--karten-hintergrund);
  }
  .kandidat-details {
    font-size: 0.85rem;
    color: var(--text-gedaempft);
  }
</style>
