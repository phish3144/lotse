<script lang="ts">
  import type { Auffaelligkeit } from '../brief';
  import { AUFFAELLIGKEIT_LABEL } from '../format';

  let { wert }: { wert: Auffaelligkeit } = $props();
</script>

<span class="punkt punkt--{wert}" title={AUFFAELLIGKEIT_LABEL[wert]} aria-label={AUFFAELLIGKEIT_LABEL[wert]}></span>

<style>
  .punkt {
    display: inline-block;
    width: 0.6em;
    height: 0.6em;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .punkt--ruhig {
    background: var(--farbe-ruhig);
  }
  .punkt--auffaellig {
    background: var(--farbe-auffaellig);
  }
  .punkt--ueberfaellig {
    background: var(--farbe-ueberfaellig);
  }
</style>
