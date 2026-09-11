<script lang="ts">
  import Entsperren from './lib/components/Entsperren.svelte';
  import Meldungen from './lib/components/Meldungen.svelte';
  import Schnellerfassung from './lib/components/Schnellerfassung.svelte';
  import Sprung from './lib/components/Sprung.svelte';
  import { echteDaten } from './lib/data/store';
  import { konto, update, type BeobachterBilanz } from './lib/data/tauri';
  import { datenVersion } from './lib/data/version.svelte';
  import { erfassung } from './lib/erfassung.svelte';
  import { meldungen } from './lib/meldung.svelte';
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

  // Der Ordner-Beobachter schreibt im Hintergrund. Damit das sichtbar wird, ohne dass
  // man die Seite neu lädt, hört die App auf seine Meldungen.
  $effect(() => {
    if (!echteDaten || !entsperrt) return;
    let abmelden: (() => void) | undefined;
    let entsorgt = false;
    void import('@tauri-apps/api/event').then(({ listen }) =>
      listen<BeobachterBilanz>('beobachter-bilanz', (e) => {
        datenVersion.bump();
        const b = e.payload;
        const teile = [
          b.datei_notizen ? `${b.datei_notizen} aus Dateien` : '',
          b.git_notizen ? `${b.git_notizen} aus Git` : '',
          b.kandidaten ? `${b.kandidaten} neue Kandidaten` : '',
        ].filter(Boolean);
        if (teile.length > 0) meldungen.zeigen(`Beobachter: ${teile.join(', ')}.`);
      }).then((un) => {
        if (entsorgt) un();
        else abmelden = un;
      }),
    );
    return () => {
      entsorgt = true;
      abmelden?.();
    };
  });

  // Nach dem Entsperren einmal nachsehen, ob es eine neuere Version gibt. Die Hülle
  // entscheidet, ob daraus wirklich eine Anfrage wird: nur wenn eingeschaltet und der
  // letzte Blick mehr als einen Tag her ist.
  let neueVersion: { version: string; seite: string } | null = $state(null);

  $effect(() => {
    if (!echteDaten || !entsperrt) return;
    let gilt = true;
    update
      .pruefen()
      .then((u) => {
        if (gilt && u.neu && u.seite) neueVersion = { version: u.neu, seite: u.seite };
      })
      .catch(() => {});
    return () => {
      gilt = false;
    };
  });

  // Auto-Lock: nach Untätigkeit schließt sich der Tresor von selbst. Das Bedrohungsmodell
  // führt das unter „Blick über die Schulter" auf – ohne diesen Zähler stand die Zusage
  // ohne Deckung da.
  //
  // Gezählt wird im Vordergrund, nicht in der Hülle: nur hier ist zu sehen, ob jemand
  // tippt. Gesperrt wird dann über dasselbe Kommando wie der Knopf in den Einstellungen,
  // und damit endet auch der Ordner-Beobachter – gesperrt heißt gesperrt.
  const WARNUNG_S = 30;
  let autoLockMinuten = $state(0);
  let warnungLaeuft = $state(0);
  let letzteRegung = $state(0);

  function regung() {
    letzteRegung = performance.now();
    if (warnungLaeuft > 0) warnungLaeuft = 0;
  }

  $effect(() => {
    if (!echteDaten || !entsperrt) return;
    let gilt = true;
    konto
      .autoLock()
      .then((m) => {
        if (gilt) autoLockMinuten = m;
      })
      .catch(() => {});
    return () => {
      gilt = false;
    };
  });

  $effect(() => {
    if (!echteDaten || !entsperrt || autoLockMinuten <= 0) return;
    letzteRegung = performance.now();
    warnungLaeuft = 0;
    const grenzeMs = autoLockMinuten * 60_000;

    const uhr = setInterval(() => {
      const ruhtSeit = performance.now() - letzteRegung;
      const restS = Math.ceil((grenzeMs - ruhtSeit) / 1000);
      if (restS <= 0) {
        // Den Zähler nicht abräumen: schlüge das Sperren fehl, käme es sonst nie
        // wieder dazu. Stattdessen von vorn zählen und es erneut versuchen.
        letzteRegung = performance.now();
        warnungLaeuft = 0;
        void konto
          .sperren()
          .then(() => {
            entsperrt = false;
          })
          .catch(() => {});
      } else if (restS <= WARNUNG_S) {
        warnungLaeuft = restS;
      }
    }, 1000);

    return () => clearInterval(uhr);
  });

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

<svelte:window
  onkeydown={aufTaste}
  onmouseup={aufMaus}
  onmousemove={regung}
  onkeypress={regung}
  onwheel={regung}
  onpointerdown={regung}
/>

{#if !entsperrt}
  <Entsperren fertig={() => (entsperrt = true)} />
{:else}
  <div class="app-geruest">
    <header class="kopfzeile">
      <div class="links">
        {#if router.kannZurueck}
          <button type="button" class="zurueck" onclick={zurueck} title="Zurück (Alt+←)" aria-label="Zurück">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m14 6-6 6 6 6" /></svg>
          </button>
        {/if}
        <a class="logo" href="#/">
          <!-- Dieselbe Marke wie das App-Icon (src-tauri/icons/icon.svg), inline:
               CLAUDE.md erlaubt keine Fremdressourcen, und eine Datei zu laden wäre
               für vier Formen Verschwendung. -->
          <svg viewBox="0 0 256 256" width="22" height="22" aria-hidden="true">
            <rect width="256" height="256" rx="56" fill="#14304a" />
            <g fill="none" stroke="#2e5f7a" stroke-width="10" opacity="0.75">
              <path d="M-10 200 C 40 180, 70 220, 120 200 S 200 180, 270 205" />
              <path d="M-10 222 C 40 202, 70 242, 120 222 S 200 202, 270 227" />
            </g>
            <path d="M80 56h32v112h64v32H80z" fill="#f2b544" />
            <circle cx="184" cy="72" r="20" fill="#e8e1d1" />
          </svg>
          Lotse
        </a>
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
    {#if warnungLaeuft > 0}
      <div class="update-band sperr-band" role="status">
        <span>Lotse sperrt in {warnungLaeuft} Sekunden.</span>
        <button type="button" class="schlicht" onclick={regung}>Wach bleiben</button>
      </div>
    {/if}
    {#if neueVersion}
      <div class="update-band">
        <span>Version {neueVersion.version} ist da.</span>
        <a href="#/einstellungen" onclick={() => (neueVersion = null)}>Ansehen</a>
        <button type="button" class="schlicht" onclick={() => (neueVersion = null)} aria-label="Hinweis schließen">
          ✕
        </button>
      </div>
    {/if}
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
    max-width: 82rem;
    margin: 0 auto;
    padding: 0 1.5rem 3rem;
  }
  .update-band {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: -0.5rem 0 1.25rem;
    padding: 0.55rem 0.9rem;
    border: 1px solid var(--rahmen);
    border-radius: 0.5rem;
    background: var(--karten-hintergrund);
    font-size: 0.9rem;
  }
  .update-band button {
    margin-left: auto;
  }
  .sperr-band {
    border-color: var(--warnung, var(--rahmen));
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
    display: inline-flex;
    align-items: center;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-gedaempft);
    border-radius: 0.5rem;
    padding: 0.3rem 0.4rem;
  }
  .zurueck:hover {
    background: var(--flaeche-still);
    color: var(--fg);
  }
  .logo {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 700;
    font-size: 1.1rem;
    letter-spacing: -0.015em;
    color: inherit;
    text-decoration: none;
  }
  nav {
    display: flex;
    gap: 0.15rem;
  }
  /* Reiter statt Textlinks: die aktive Seite trägt eine Fläche, keine Farbe –
     Bernstein und Akzent bleiben den Handlungen und Zuständen vorbehalten. */
  nav a {
    color: var(--text-gedaempft);
    text-decoration: none;
    font-size: 0.85rem;
    padding: 0.35rem 0.65rem;
    border-radius: 0.45rem;
  }
  nav a:hover {
    color: var(--fg);
    background: var(--flaeche-still);
  }
  nav a.aktiv {
    color: var(--fg);
    font-weight: 600;
    background: var(--flaeche-still);
  }
  .aktionen {
    display: flex;
    gap: 0.4rem;
  }
  .werkzeug {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    border: 1px solid var(--rahmen);
    background: var(--hintergrund);
    color: var(--text-gedaempft);
    border-radius: 0.5rem;
    padding: 0.38rem 0.7rem;
    font-size: 0.85rem;
    white-space: nowrap;
  }
  .werkzeug:hover {
    color: var(--fg);
    border-color: var(--akzent);
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
