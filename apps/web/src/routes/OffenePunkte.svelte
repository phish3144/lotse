<script lang="ts">
  import NoteText from '../lib/components/NoteText.svelte';
  import { provider } from '../lib/data/store';
  import { datenVersion } from '../lib/data/version.svelte';
  import { alterInTagenText } from '../lib/format';
  import { meldungen } from '../lib/meldung.svelte';

  async function laden() {
    const [faeden, projekte] = await Promise.all([provider.listOpenThreads(), provider.listProjects()]);
    const titelNachId = new Map(projekte.map((p) => [p.id, p.titel] as const));
    return faeden.map((faden) => ({ faden, projektTitel: titelNachId.get(faden.projekt_id) ?? faden.projekt_id }));
  }

  let datenPromise = $derived.by(() => {
    datenVersion.wert;
    return laden();
  });

  let fehlerText: string | null = $state(null);
  let laeuft: string | null = $state(null);

  async function erledigen(notizId: string) {
    laeuft = notizId;
    fehlerText = null;
    try {
      await provider.completeThread(notizId);
      datenVersion.bump();
      meldungen.zeigen('Faden abgehakt.');
    } catch (e) {
      fehlerText = e instanceof Error ? e.message : String(e);
    } finally {
      laeuft = null;
    }
  }
</script>

<svelte:head><title>Offene Punkte · Lotse</title></svelte:head>

<h1>Offene Punkte</h1>
<p class="hinweis">Alle offenen Fäden, projektübergreifend – die einzige Aufgabenliste, die Lotse braucht.</p>

{#if fehlerText}
  <p class="hinweis fehler">{fehlerText}</p>
{/if}

{#await datenPromise}
  <p class="hinweis">Lade offene Punkte…</p>
{:then eintraege}
  {#if eintraege.length === 0}
    <p class="hinweis">Keine offenen Fäden.</p>
  {:else}
    <ul class="faeden-liste">
      {#each eintraege as { faden, projektTitel } (faden.id)}
        <li class="faden">
          <div class="faden-kopf">
            <a class="projekt-link" href={`#/projekt/${faden.projekt_id}`}>{projektTitel}</a>
            <span class="alter">{alterInTagenText(faden.ts)}</span>
          </div>
          <div class="faden-koerper">
            <button
              type="button"
              class="haken"
              title="Faden abhaken"
              aria-label="Faden abhaken"
              onclick={() => erledigen(faden.id)}
              disabled={laeuft === faden.id}
            >
              ✓
            </button>
            <NoteText text={faden.text} />
          </div>
        </li>
      {/each}
    </ul>
  {/if}
{:catch fehler}
  <p class="hinweis fehler">Offene Punkte konnten nicht geladen werden: {fehler.message}</p>
{/await}

<style>
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
  .faeden-liste {
    list-style: none;
    margin: 1.5rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .faden {
    padding: 0.75rem 1rem;
    border: 1px solid var(--rahmen);
    border-radius: 0.6rem;
    background: var(--karten-hintergrund);
  }
  .faden-kopf {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 0.75rem;
    margin-bottom: 0.35rem;
  }
  .projekt-link {
    font-weight: 600;
    color: var(--akzent);
    text-decoration: none;
  }
  .projekt-link:hover {
    text-decoration: underline;
  }
  .alter {
    font-size: 0.8rem;
    color: var(--text-gedaempft);
    white-space: nowrap;
  }
  .faden-koerper {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
  }
  .haken {
    flex: none;
    border: 1px solid var(--rahmen);
    background: transparent;
    color: var(--farbe-ruhig);
    border-radius: 0.35rem;
    padding: 0 0.45rem;
    line-height: 1.5;
  }
  .haken:hover:not(:disabled) {
    border-color: var(--farbe-ruhig);
  }
  .haken:disabled {
    opacity: 0.5;
  }
</style>
