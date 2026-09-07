<script lang="ts">
  // Sucht beim Tippen. Die Adresse wird dabei nur ersetzt, nicht fortgeschrieben,
  // sonst führt jeder Tastendruck einen Schritt in den Zurück-Weg ein.
  import NoteText from '../lib/components/NoteText.svelte';
  import { provider } from '../lib/data/store';
  import { alterInTagenText, PRUEFSTATUS_LABEL } from '../lib/format';
  import { ersetze } from '../lib/router.svelte';

  let { query }: { query: string } = $props();

  const VERZOEGERUNG_MS = 180;

  let eingabe = $state('');
  let begriff = $state('');
  let inputEl: HTMLInputElement | undefined = $state();
  let timer: ReturnType<typeof setTimeout> | undefined;
  // Zuletzt aus der Adresse übernommener Begriff. Bewusst kein $state: dient nur dazu,
  // eigene Adressänderungen von fremden (Link, Zurück-Weg) zu unterscheiden.
  let ausAdresse: string | null = null;

  $effect(() => {
    if (query !== ausAdresse) {
      ausAdresse = query;
      eingabe = query;
      begriff = query.trim();
    }
  });

  // Beim Betreten der Seite steht der Fokus im Feld; die Suche ist ein Tastaturwerkzeug.
  $effect(() => {
    inputEl?.focus();
  });

  function beiEingabe() {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      begriff = eingabe.trim();
      ausAdresse = begriff;
      ersetze(begriff ? `#/suche?q=${encodeURIComponent(begriff)}` : '#/suche');
    }, VERZOEGERUNG_MS);
  }

  $effect(() => () => {
    if (timer) clearTimeout(timer);
  });

  let ergebnisPromise = $derived(begriff ? provider.search(begriff) : Promise.resolve(null));
</script>

<svelte:head><title>Suche · Lotse</title></svelte:head>

<h1>Suche</h1>
<div class="suchform">
  <input
    bind:this={inputEl}
    bind:value={eingabe}
    oninput={beiEingabe}
    type="search"
    placeholder="Projekte, Logbuch, Referenzen, Tresor-Titel …"
    aria-label="Suche"
  />
</div>

{#if !begriff}
  <p class="hinweis">Tippen genügt – gesucht wird über alle Projekte, Logbücher, Referenzen und Tresor-Titel.</p>
{:else}
  {#await ergebnisPromise}
    <p class="hinweis">Suche läuft…</p>
  {:then ergebnis}
    {#if !ergebnis}
      <p class="hinweis">Tippen genügt.</p>
    {:else}
    {@const gesamt = ergebnis.projects.length + ergebnis.notes.length + ergebnis.references.length + ergebnis.vaultEntries.length}
    {#if gesamt === 0}
      <p class="hinweis">Keine Treffer für „{begriff}“.</p>
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
              <li><a href="#/tresor">{v.titel}</a></li>
            {/each}
          </ul>
        </section>
      {/if}
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
