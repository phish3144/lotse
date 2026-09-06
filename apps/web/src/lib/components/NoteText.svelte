<script lang="ts">
  // Rendert Notiztext als reinen Text mit Absatz- und Zeilenumbrüchen – kein
  // Markdown, kein {@html}, siehe THREAT_MODEL.md Abschnitt 5. Der Notizkern
  // speichert Markdown, aber solange kein sanitisierender Renderer existiert,
  // zeigen wir bewusst nur Klartext.
  let { text }: { text: string } = $props();

  const absaetze = $derived(
    text
      .split(/\n{2,}/)
      .map((abs) => abs.split('\n'))
      .filter((zeilen) => zeilen.some((z) => z.trim().length > 0)),
  );
</script>

<div class="notiztext">
  {#each absaetze as zeilen, absatzIndex (absatzIndex)}
    <p>
      {#each zeilen as zeile, zeilenIndex (zeilenIndex)}
        {zeile}{#if zeilenIndex < zeilen.length - 1}<br />{/if}
      {/each}
    </p>
  {/each}
</div>

<style>
  .notiztext p {
    margin: 0 0 0.5em;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .notiztext p:last-child {
    margin-bottom: 0;
  }
</style>
