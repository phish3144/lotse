<script lang="ts">
  import Entsperren from './lib/components/Entsperren.svelte';
  import Meldungen from './lib/components/Meldungen.svelte';
  import Schnellerfassung from './lib/components/Schnellerfassung.svelte';
  import Sprung from './lib/components/Sprung.svelte';
  import { echteDaten } from './lib/data/store';
  import { erfassung } from './lib/erfassung.svelte';
  import { router, zurueck } from './lib/router.svelte';
  import { sprung } from './lib/sprung.svelte';
  import Einstellungen from './routes/Einstellungen.svelte';
  import Hafen from './routes/Hafen.svelte';
  import OffenePunkte from './routes/OffenePunkte.svelte';
  import ProjektSeite from './routes/ProjektSeite.svelte';
  import Suche from './routes/Suche.svelte';
  import Tresor from './routes/Tresor.svelte';

  // In der Tauri-Hülle steht vor allem der Entsperr-Bildschirm; im Browser mit
  // Beispieldaten entfällt er.
  let entsperrt = $state(!echteDaten);

  function aufTaste(e: KeyboardEvent) {
    // Strg/Cmd+P springt zum Projekt. Das Drucken-Kürzel hat in dieser App keinen Sinn.
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'p') {
      e.preventDefault();
      sprung.umschalten();
    } else if (e.key === 'Escape' && sprung.offen) {
      sprung.schliessen();
    } else if (e.altKey && e.key === 'ArrowLeft') {
      e.preventDefault();
      zurueck();
    }
  }

  // Die Zurück-Taste der Maus; im Tauri-Fenster gibt es sonst keinen Weg zurück.
  function aufMaus(e: MouseEvent) {
    if (e.button === 3) {
      e.preventDefault();
      zurueck();
    }
  }
</script>

<svelte:window onkeydown={aufTaste} onmouseup={aufMaus} />

{#if !entsperrt}
  <Entsperren fertig={() => (entsperrt = true)} />
{:else}
  <div class="app-geruest">
    <header class="kopfzeile">
      <div class="links">
        {#if router.kannZurueck}
          <button type="button" class="zurueck" onclick={zurueck} title="Zurück (Alt+←)" aria-label="Zurück">←</button>
        {/if}
        <a class="logo" href="#/">Lotse</a>
      </div>
      <nav>
        <a href="#/" class:aktiv={router.current.segmente.length === 0}>Hafen</a>
        <a href="#/offen" class:aktiv={router.current.segmente[0] === 'offen'}>Offene Punkte</a>
        <a href="#/tresor" class:aktiv={router.current.segmente[0] === 'tresor'}>Tresor</a>
        <a href="#/suche" class:aktiv={router.current.segmente[0] === 'suche'}>Suche</a>
        <a href="#/einstellungen" class:aktiv={router.current.segmente[0] === 'einstellungen'}>Einstellungen</a>
      </nav>
      <div class="aktionen">
        <button type="button" class="werkzeug" onclick={() => sprung.oeffnen()} title="Zu Projekt springen (Strg+P)">
          Springen <kbd>⌘P</kbd>
        </button>
        <button type="button" class="werkzeug" onclick={() => erfassung.oeffnen()} title="Schnellerfassung (Strg+K)">
          Erfassen <kbd>⌘K</kbd>
        </button>
      </div>
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
  <Sprung />
  <Meldungen />
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
  .links {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .zurueck {
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.15rem 0.55rem;
    font-size: 1rem;
    line-height: 1.4;
  }
  .zurueck:hover {
    border-color: var(--akzent);
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
  .aktionen {
    display: flex;
    gap: 0.4rem;
  }
  .werkzeug {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: inherit;
    border-radius: 0.4rem;
    padding: 0.35rem 0.7rem;
    white-space: nowrap;
  }
  .werkzeug:hover {
    border-color: var(--akzent);
  }
  .werkzeug kbd {
    font-family: inherit;
    font-size: 0.75rem;
    color: var(--text-gedaempft);
    border: 1px solid var(--rahmen);
    border-radius: 0.25rem;
    padding: 0.02em 0.3em;
  }
  @media (max-width: 60rem) {
    .werkzeug kbd {
      display: none;
    }
  }
  @media (max-width: 46rem) {
    .kopfzeile {
      flex-wrap: wrap;
    }
    nav {
      order: 3;
      width: 100%;
      gap: 0.9rem;
      font-size: 0.9rem;
      overflow-x: auto;
    }
  }
</style>
