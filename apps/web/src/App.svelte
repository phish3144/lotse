<script lang="ts">
  import Entsperren from './lib/components/Entsperren.svelte';
  import Schnellerfassung from './lib/components/Schnellerfassung.svelte';
  import { echteDaten } from './lib/data/store';
  import { erfassung } from './lib/erfassung.svelte';
  import { router } from './lib/router.svelte';
  import Einstellungen from './routes/Einstellungen.svelte';
  import Hafen from './routes/Hafen.svelte';
  import OffenePunkte from './routes/OffenePunkte.svelte';
  import ProjektSeite from './routes/ProjektSeite.svelte';
  import Suche from './routes/Suche.svelte';
  import Tresor from './routes/Tresor.svelte';

  // In der Tauri-Hülle steht vor allem der Entsperr-Bildschirm; im Browser mit
  // Beispieldaten entfällt er.
  let entsperrt = $state(!echteDaten);
</script>

{#if !entsperrt}
  <Entsperren fertig={() => (entsperrt = true)} />
{:else}
<div class="app-geruest">
  <header class="kopfzeile">
    <a class="logo" href="#/">Lotse</a>
    <nav>
      <a href="#/" class:aktiv={router.current.segmente.length === 0}>Hafen</a>
      <a href="#/offen" class:aktiv={router.current.segmente[0] === 'offen'}>Offene Punkte</a>
      <a href="#/tresor" class:aktiv={router.current.segmente[0] === 'tresor'}>Tresor</a>
      <a href="#/suche" class:aktiv={router.current.segmente[0] === 'suche'}>Suche</a>
      <a href="#/einstellungen" class:aktiv={router.current.segmente[0] === 'einstellungen'}>Einstellungen</a>
    </nav>
    <button type="button" class="erfassen" onclick={() => erfassung.oeffnen()} title="Schnellerfassung (Strg+K)">
      Erfassen <kbd>⌘K</kbd>
    </button>
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
    {:else if router.current.segmente[0] === 'tresor'}
      <Tresor />
    {:else if router.current.segmente[0] === 'suche'}
      <Suche query={router.current.query.q ?? ''} />
    {:else if router.current.segmente[0] === 'einstellungen'}
      <Einstellungen />
    {:else}
      <p>Seite nicht gefunden.</p>
    {/if}
  </main>
</div>
<Schnellerfassung />
{/if}

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
  .erfassen {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.35rem 0.7rem;
    white-space: nowrap;
  }
  .erfassen:hover {
    border-color: var(--akzent);
  }
  .erfassen kbd {
    font-family: inherit;
    font-size: 0.75rem;
    color: var(--text-gedaempft);
    border: 1px solid var(--rahmen);
    border-radius: 0.25rem;
    padding: 0.02em 0.3em;
  }
  @media (max-width: 40rem) {
    .kopfzeile {
      flex-wrap: wrap;
    }
    nav {
      order: 3;
      width: 100%;
      gap: 0.9rem;
      font-size: 0.9rem;
    }
  }
</style>
