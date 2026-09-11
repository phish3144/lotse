<script lang="ts">
  // Tresor-Einträge mit verdeckten Werten. Ein Wert wird genau auf Klick entschlüsselt,
  // nie auf Vorrat geladen und nach kurzer Zeit von selbst wieder verdeckt
  // (THREAT_MODEL.md: der Klartext soll so kurz wie möglich am Bildschirm stehen).
  import { provider } from '../data/store';
  import { datenVersion } from '../data/version.svelte';
  import { meldungen } from '../meldung.svelte';
  import { STUFE_LABEL } from '../format';
  import { ablageLeeren as leeren } from '../zwischenablage';
  import type { Id, Projekt, TresorEintrag } from '../data/types';

  let {
    eintraege,
    projekte = [],
    zeigeProjekte = false,
  }: { eintraege: TresorEintrag[]; projekte?: Projekt[]; zeigeProjekte?: boolean } = $props();

  /** Sekunden, nach denen ein aufgedeckter Wert wieder verschwindet. */
  const VERDECKEN_NACH_S = 30;
  /** Sekunden, nach denen ein kopierter Wert aus der Zwischenablage verschwindet. */
  const ZWISCHENABLAGE_NACH_S = 20;

  let aufgedeckt: { schluessel: string; wert: string } | null = $state(null);
  let laeuft: string | null = $state(null);
  let fehler: string | null = $state(null);
  let kopiert = $state(false);
  /** Läuft, solange der Wert noch in der Zwischenablage steht. */
  let kopierRest = $state(0);
  /** Was gerade in der Zwischenablage steht – unabhängig davon, ob es noch sichtbar ist. */
  let inAblage: string | null = null;
  let loeschKandidat: Id | null = $state(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let ablageUhr: ReturnType<typeof setInterval> | undefined;

  function schluesselVon(id: Id, feld: string) {
    return `${id}|${feld}`;
  }

  function verdecken() {
    aufgedeckt = null;
    kopiert = false;
    if (timer) clearTimeout(timer);
  }

  async function aufdecken(id: Id, feld: string) {
    const schluessel = schluesselVon(id, feld);
    if (aufgedeckt?.schluessel === schluessel) {
      verdecken();
      return;
    }
    laeuft = schluessel;
    fehler = null;
    kopiert = false;
    try {
      const wert = await provider.readVaultField(id, feld);
      aufgedeckt = { schluessel, wert };
      if (timer) clearTimeout(timer);
      timer = setTimeout(verdecken, VERDECKEN_NACH_S * 1000);
    } catch (e) {
      fehler = e instanceof Error ? e.message : String(e);
    } finally {
      laeuft = null;
    }
  }

  /** Leert die Zwischenablage; das Wie steht in `lib/zwischenablage.ts`. */
  async function ablageLeeren(wert: string) {
    if (inAblage === wert) inAblage = null;
    if (!navigator.clipboard) return;
    await leeren(wert, navigator.clipboard);
  }

  async function kopieren() {
    if (!aufgedeckt) return;
    const wert = aufgedeckt.wert;
    try {
      await navigator.clipboard.writeText(wert);
      kopiert = true;
      inAblage = wert;
      if (ablageUhr) clearInterval(ablageUhr);
      kopierRest = ZWISCHENABLAGE_NACH_S;
      ablageUhr = setInterval(() => {
        kopierRest -= 1;
        if (kopierRest <= 0) {
          if (ablageUhr) clearInterval(ablageUhr);
          ablageUhr = undefined;
          kopiert = false;
          void ablageLeeren(wert);
        }
      }, 1000);
    } catch {
      // Ohne Zwischenablage bleibt der Wert lesbar am Bildschirm stehen.
      fehler = 'Zwischenablage nicht verfügbar. Wert von Hand übernehmen.';
    }
  }

  async function loeschen(id: Id) {
    fehler = null;
    try {
      await provider.deleteVaultEntry(id);
      loeschKandidat = null;
      verdecken();
      datenVersion.bump();
      meldungen.zeigen('Eintrag gelöscht, Schlüssel vernichtet.');
    } catch (e) {
      fehler = e instanceof Error ? e.message : String(e);
    }
  }

  function projektNamen(ids: Id[]): string {
    return ids
      .map((id) => projekte.find((p) => p.id === id)?.titel ?? '—')
      .filter(Boolean)
      .join(', ');
  }

  // Beim Verlassen der Ansicht: Zähler abräumen und, falls noch etwas in der
  // Zwischenablage steht, auch die leeren. Sonst überlebte das Passwort den Wechsel.
  $effect(() => () => {
    if (timer) clearTimeout(timer);
    if (ablageUhr) {
      clearInterval(ablageUhr);
      ablageUhr = undefined;
    }
    if (inAblage) void ablageLeeren(inAblage);
  });
</script>

{#if fehler}
  <p class="fehler">{fehler}</p>
{/if}

{#if eintraege.length === 0}
  <p class="hinweis">Keine Zugänge hinterlegt.</p>
{:else}
  <ul class="tresor-liste">
    {#each eintraege as eintrag (eintrag.id)}
      <li class:nur-desktop={eintrag.stufe === 'nur_desktop'}>
        <div class="kopf">
          <strong>{eintrag.titel}</strong>
          <span class="badge badge--stufe-{eintrag.stufe}">{STUFE_LABEL[eintrag.stufe]}</span>
          {#if zeigeProjekte && eintrag.projekt_ids.length > 0}
            <span class="projekt-chip">{projektNamen(eintrag.projekt_ids)}</span>
          {/if}
          {#if loeschKandidat === eintrag.id}
            <span class="loesch-frage">
              Endgültig löschen?
              <button type="button" class="gefahr" onclick={() => loeschen(eintrag.id)}>Ja, vernichten</button>
              <button type="button" onclick={() => (loeschKandidat = null)}>Abbrechen</button>
            </span>
          {:else}
            <button type="button" class="loeschen" onclick={() => (loeschKandidat = eintrag.id)}>Löschen</button>
          {/if}
        </div>

        <ul class="felder">
          {#each eintrag.felder as feld (feld.name)}
            {@const schluessel = schluesselVon(eintrag.id, feld.name)}
            {@const offen = aufgedeckt?.schluessel === schluessel}
            <li class="feld">
              <span class="feld-name">{feld.name}</span>
              <code class="wert" class:offen>{offen ? aufgedeckt?.wert : '••••••••'}</code>
              <button type="button" onclick={() => aufdecken(eintrag.id, feld.name)} disabled={laeuft === schluessel}>
                {laeuft === schluessel ? '…' : offen ? 'Verbergen' : 'Zeigen'}
              </button>
              {#if offen}
                <button type="button" onclick={kopieren}>
                  {kopiert ? `Kopiert – noch ${kopierRest} s` : 'Kopieren'}
                </button>
                <span class="hinweis auto">verdeckt sich von selbst</span>
              {/if}
            </li>
          {/each}
        </ul>
      </li>
    {/each}
  </ul>
{/if}

<style>
  /* Ein Raster statt einer Spalte: Zugänge sind kurz und nebeneinander schneller zu
     überfliegen. Auf der Projektseite steht nur einer je Projekt – dort bleibt es
     eine Spalte, weil die Nebenspalte schmal ist. */
  .tresor-liste {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
    gap: 0.7rem;
  }
  .tresor-liste > li {
    border: 1px solid var(--rahmen);
    border-radius: 0.7rem;
    background: var(--karten-hintergrund);
    padding: 0.75rem 0.9rem;
    box-shadow: var(--schatten);
    min-width: 0;
  }
  /* „nur Desktop“ ist keine Beschriftung, sondern eine Eigenschaft des Eintrags –
     deshalb auch an der Kante ablesbar, ohne zu lesen. */
  .tresor-liste > li.nur-desktop {
    border-left: 3px solid var(--akzent);
  }
  .kopf {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    min-width: 0;
  }
  .kopf strong {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .projekt-chip {
    font-size: 0.7rem;
    background: var(--flaeche-still);
    border-radius: 999px;
    padding: 0.1rem 0.5rem;
    color: var(--text-gedaempft);
  }
  .kopf .loeschen,
  .loesch-frage {
    margin-left: auto;
  }
  .felder {
    list-style: none;
    margin: 0.6rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }
  .feld {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    font-size: 0.85rem;
    padding: 0.3rem 0;
    border-top: 1px solid var(--rahmen-still);
  }
  .feld:first-child {
    border-top: none;
  }
  .feld-name {
    color: var(--text-gedaempft);
    min-width: 6rem;
    font-size: 0.8rem;
  }
  .wert {
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    background: var(--hintergrund);
    border-radius: 0.3rem;
    padding: 0.15rem 0.45rem;
    overflow-wrap: anywhere;
  }
  .wert.offen {
    border: 1px solid var(--farbe-auffaellig);
  }
  button {
    border: 1px solid var(--rahmen);
    background: transparent;
    color: inherit;
    border-radius: 0.35rem;
    padding: 0.15rem 0.6rem;
    font-size: 0.8rem;
  }
  button:hover:not(:disabled) {
    border-color: var(--akzent);
  }
  button.gefahr {
    color: var(--farbe-ueberfaellig);
    border-color: var(--farbe-ueberfaellig);
  }
  .loesch-frage {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
    color: var(--farbe-ueberfaellig);
  }
  .badge {
    font-size: 0.75rem;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    border: 1px solid var(--rahmen);
    white-space: nowrap;
  }
  /* Umgedreht: „überall“ ist der Normalfall und bleibt still; die Einschränkung
     „nur Desktop“ trägt die Farbe – dieselbe wie die linke Kante der Karte, damit
     Etikett und Kante dasselbe sagen. */
  .badge--stufe-ueberall {
    color: var(--text-gedaempft);
    border-color: var(--rahmen);
  }
  .badge--stufe-nur_desktop {
    color: var(--akzent);
    border-color: var(--akzent);
  }
  .hinweis {
    color: var(--text-gedaempft);
  }
  .projekte,
  .auto {
    font-size: 0.8rem;
  }
  .fehler {
    color: var(--farbe-ueberfaellig);
    font-size: 0.9rem;
  }
</style>
