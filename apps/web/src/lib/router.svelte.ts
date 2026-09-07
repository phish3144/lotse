// Minimaler Hash-Router ohne externe Abhängigkeit. Unterstützte Formen:
// "#/", "#/projekt/:id", "#/offen", "#/tresor", "#/suche?q=...", "#/einstellungen".
//
// Die Tauri-Hülle hat keine Browserleiste: ohne eigenen Zurück-Weg käme man aus einem
// Projekt nur über die Kopfzeile heraus. Deshalb führt der Router selbst Buch darüber,
// wie tief man in der App steckt.

export interface Route {
  pfad: string;
  segmente: string[];
  query: Record<string, string>;
}

function parseHash(hash: string): Route {
  const roh = hash.startsWith('#') ? hash.slice(1) : hash;
  const [pfad, queryString = ''] = roh.split('?');
  const query = Object.fromEntries(new URLSearchParams(queryString));
  const segmente = pfad.split('/').filter(Boolean);
  return { pfad: pfad || '/', segmente, query };
}

class Router {
  private hash = $state(typeof window !== 'undefined' ? window.location.hash || '#/' : '#/');
  /** Wie viele eigene Schritte tief; nur darüber gibt es einen sinnvollen Zurück-Weg. */
  tiefe = $state(0);

  get current(): Route {
    return parseHash(this.hash);
  }

  get kannZurueck(): boolean {
    return this.tiefe > 0;
  }

  sync(hash: string) {
    this.hash = hash || '#/';
  }
}

export const router = new Router();

// Die meisten Wege in die Tiefe sind gewöhnliche Links (Projektkarten, Suchtreffer).
// Deshalb wird am hashchange gezählt und nicht beim Aufruf – sonst bliebe der
// Zurück-Weg für genau die Wege aus, die man am häufigsten geht.
let gehtZurueck = false;

if (typeof window !== 'undefined') {
  window.addEventListener('hashchange', () => {
    if (gehtZurueck) {
      gehtZurueck = false;
    } else {
      router.tiefe += 1;
    }
    router.sync(window.location.hash);
  });
  if (!window.location.hash) {
    window.location.hash = '#/';
  }
}

export function navigiereZu(pfad: string) {
  if (pfad === window.location.hash) return;
  window.location.hash = pfad;
}

/** Einen Schritt zurück. Auf der obersten Ebene bleibt es beim Hafen. */
export function zurueck() {
  if (router.tiefe > 0) {
    gehtZurueck = true;
    router.tiefe -= 1;
    window.history.back();
  } else if (window.location.hash !== '#/') {
    window.location.hash = '#/';
  }
}

/** Ersetzt den aktuellen Eintrag, ohne die Tiefe zu erhöhen (z. B. Suche beim Tippen). */
export function ersetze(pfad: string) {
  if (typeof window === 'undefined' || pfad === window.location.hash) return;
  window.history.replaceState(null, '', pfad);
  router.sync(pfad);
}
