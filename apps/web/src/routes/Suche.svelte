<script lang="ts">
  import NoteText from '../lib/components/NoteText.svelte';
  import { provider } from '../lib/data/store';
  import { alterInTagenText, PRUEFSTATUS_LABEL } from '../lib/format';
  import { navigiereZu } from '../lib/router.svelte';

  let { query }: { query: string } = $props();

  let eingabe = $state('');
  $effect(() => {
    eingabe = query;
  });

  let ergebnisPromise = $derived(provider.search(query));

  function suchen(e: SubmitEvent) {
    e.preventDefault();
    navigiereZu(`#/suche?q=${encodeURIComponent(eingabe)}`);
  }
</script>

<svelte:head><title>Suche · Lotse</title></svelte:head>

<h1>Suche</h1>
<form class="suchform" onsubmit={suchen}>
  <input type="search" placeholder="Projekte, Logbuch, Referenzen, Tresor-Titel …" bind:value={eingabe} aria-label="Suche" />
  <button type="submit">Suchen</button>
</form>

{#if !query.trim()}
  <p class="hinweis">Suchbegriff eingeben.</p>
{:else}
  {#await ergebnisPromise}
    <p class="hinweis">Suche läuft…</p>
  {:then ergebnis}
    {@const gesamt = ergebnis.projects.length + ergebnis.notes.length + ergebnis.references.length + ergebnis.vaultEntries.length}
    {#if gesamt === 0}
      <p class="hinweis">Keine Treffer für „{query}“.</p>
    {:else}
      {#if ergebnis.projects.length > 0}
        <section>
          <h2>Projekte</h2>
          <ul class="treffer-liste">
            {#each ergebnis.projects as p (p.id)}
              <li><a href={`#/projekt/${p.id}`}><strong>{p.titel}</strong> — {p.kurs}</a></li>
            {/each}
          </ul>
        </section>
      {/if}
      {#if ergebnis.notes.length > 0}
        <section>
          <h2>Logbuch</h2>
          <ul class="treffer-liste">
            {#each ergebnis.notes as n (n.id)}
              <li>
                <a class="notiz-treffer" href={`#/projekt/${n.projekt_id}`}>
                  <span class="alter">{alterInTagenText(n.ts)}</span>
                  <NoteText text={n.text} />
                </a>
              </li>
            {/each}
          </ul>
        </section>
      {/if}
      {#if ergebnis.references.length > 0}
        <section>
          <h2>Referenzen</h2>
          <ul class="treffer-liste">
            {#each ergebnis.references as r (r.id)}
              <li>
                <a href={`#/projekt/${r.projekt_id}`}>{r.ziel}</a>
                <span class="hinweis">({PRUEFSTATUS_LABEL[r.pruefstatus]})</span>
              </li>
            {/each}
          </ul>
        </section>
      {/if}
      {#if ergebnis.vaultEntries.length > 0}
        <section>
          <h2>Zugänge</h2>
          <ul class="treffer-liste">
            {#each ergebnis.vaultEntries as v (v.id)}
              <li>{v.titel}</li>
            {/each}
          </ul>
        </section>
      {/if}
    {/if}
  {:catch fehler}
    <p class="hinweis fehler">Suche fehlgeschlagen: {fehler.message}</p>
  {/await}
{/if}

<style>
  .suchform {
    display: flex;
    gap: 0.5rem;
    margin: 1rem 0 1.5rem;
    max-width: 32rem;
  }
  .suchform input {
    flex: 1;
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
  }
  section {
    margin-bottom: 1.5rem;
  }
  h2 {
    font-size: 1rem;
    margin: 0 0 0.5rem;
  }
  .treffer-liste {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .treffer-liste a {
    color: var(--akzent);
    text-decoration: none;
  }
  .treffer-liste a:hover {
    text-decoration: underline;
  }
  .notiz-treffer {
    display: block;
  }
  .notiz-treffer .alter {
    display: block;
    font-size: 0.8rem;
    color: var(--text-gedaempft);
    margin-bottom: 0.15rem;
  }
</style>
