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
    font-size: 0.88rem;
    text-align: left;
    max-width: min(30rem, 92vw);
    padding: 0.55rem 0.95rem;
    border-radius: 0.6rem;
    border: 1px solid var(--rahmen);
    background: var(--karten-hintergrund);
    color: var(--fg);
    box-shadow: 0 10px 30px rgba(11, 27, 43, 0.3);
    animation: auftauchen 180ms ease-out;
  }
  /* Der Zustand steht an der linken Kante, nicht als Rahmen ringsum: ein grün
     umrandeter Kasten liest sich als Warnung, eine Kante als Vermerk. */
  .meldung.gut {
    border-left: 3px solid var(--farbe-ruhig);
  }
  .meldung.fehler {
    border-left: 3px solid var(--farbe-ueberfaellig);
    color: var(--farbe-ueberfaellig);
  }
  @media (prefers-reduced-motion: reduce) {
    .meldung {
      animation: none;
    }
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
