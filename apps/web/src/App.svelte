<script lang="ts">
  import Schnellerfassung from './lib/components/Schnellerfassung.svelte';
  import { router } from './lib/router.svelte';
  import Hafen from './routes/Hafen.svelte';
  import OffenePunkte from './routes/OffenePunkte.svelte';
  import ProjektSeite from './routes/ProjektSeite.svelte';
  import Suche from './routes/Suche.svelte';
</script>

<div class="app-geruest">
  <header class="kopfzeile">
    <a class="logo" href="#/">Lotse</a>
    <nav>
      <a href="#/" class:aktiv={router.current.segmente.length === 0}>Hafen</a>
      <a href="#/offen" class:aktiv={router.current.segmente[0] === 'offen'}>Offene Punkte</a>
      <a href="#/suche" class:aktiv={router.current.segmente[0] === 'suche'}>Suche</a>
    </nav>
  </header>
  <main>
    {#if router.current.segmente.length === 0}
      <Hafen />
    {:else if router.current.segmente[0] === 'projekt' && router.current.segmente[1]}
      {#key router.current.segmente[1]}
        <ProjektSeite id={router.current.segmente[1]} />
      {/key}
    {:else if router.current.segmente[0] === 'offen'}
      <OffenePunkte />
    {:else if router.current.segmente[0] === 'suche'}
      <Suche query={router.current.query.q ?? ''} />
    {:else}
      <p>Seite nicht gefunden.</p>
    {/if}
  </main>
</div>
<Schnellerfassung />

<style>
  .app-geruest {
    max-width: 64rem;
    margin: 0 auto;
    padding: 0 1.25rem 3rem;
  }
  .kopfzeile {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1.1rem 0;
    border-bottom: 1px solid var(--rahmen);
    margin-bottom: 1.5rem;
  }
  .logo {
    font-weight: 700;
    font-size: 1.15rem;
    color: inherit;
    text-decoration: none;
  }
  nav {
    display: flex;
    gap: 1.1rem;
  }
  nav a {
    color: var(--text-gedaempft);
    text-decoration: none;
    font-size: 0.9rem;
  }
  nav a:hover {
    color: var(--fg);
  }
  nav a.aktiv {
    color: var(--akzent);
    font-weight: 600;
  }
</style>
