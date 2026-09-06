// Minimaler Hash-Router ohne externe Abhängigkeit. Unterstützte Formen:
// "#/", "#/projekt/:id", "#/offen", "#/suche?q=...".

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

  get current(): Route {
    return parseHash(this.hash);
  }

  sync(hash: string) {
    this.hash = hash || '#/';
  }
}

export const router = new Router();

if (typeof window !== 'undefined') {
  window.addEventListener('hashchange', () => router.sync(window.location.hash));
  if (!window.location.hash) {
    window.location.hash = '#/';
  }
}

export function navigiereZu(pfad: string) {
  window.location.hash = pfad;
}
