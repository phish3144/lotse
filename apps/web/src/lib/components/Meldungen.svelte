<script lang="ts">
  import { meldungen } from '../meldung.svelte';
</script>

{#if meldungen.liste.length > 0}
  <div class="meldungen" role="status" aria-live="polite">
    {#each meldungen.liste as m (m.id)}
      <button type="button" class="meldung {m.art}" onclick={() => meldungen.schliessen(m.id)} title="Ausblenden">
        {m.text}
      </button>
    {/each}
  </div>
{/if}

<style>
  .meldungen {
    position: fixed;
    bottom: 1.2rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    z-index: 200;
    align-items: center;
    pointer-events: none;
  }
  .meldung {
    pointer-events: auto;
    font: inherit;
    font-size: 0.9rem;
    text-align: left;
    max-width: min(30rem, 92vw);
    padding: 0.55rem 0.95rem;
    border-radius: 0.5rem;
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: var(--fg);
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.22);
    animation: auftauchen 180ms ease-out;
  }
  .meldung.gut {
    border-color: var(--farbe-ruhig);
  }
  .meldung.fehler {
    border-color: var(--farbe-ueberfaellig);
    color: var(--farbe-ueberfaellig);
  }
  @keyframes auftauchen {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .meldung {
      animation: none;
    }
  }
</style>
