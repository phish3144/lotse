# Abhängigkeiten: was geduldet wird und warum

Für Rust regelt das `deny.toml` mit datierten, begründeten Ausnahmen, geprüft von
`cargo deny` in CI. Für JavaScript gab es lange nichts — dieses Dokument schließt die
Lücke. Es hält fest, welche Meldungen bewusst stehenbleiben, damit dieselbe Frage nicht
alle sechs Monate neu beantwortet werden muss.

## Der Maßstab: was ausgeliefert wird

`npm audit` bewertet den **ganzen** Baum, Werkzeuge eingeschlossen. Das ist für eine
Bibliothek richtig und für uns irreführend: Vite, Vitest und Wrangler laufen auf einem
Entwicklerrechner und landen in keinem Bau.

Deshalb prüft CI in beiden npm-Projekten:

```
npm audit --omit=dev --audit-level=high
```

Das ist der Maßstab, der zählt — und er wird rot, sobald jemand eine **Laufzeit**-
Abhängigkeit mit Meldung hinzufügt.

| Projekt | Laufzeit-Abhängigkeiten | Stand |
|---|---|---|
| `services/sync-worker` | **keine** — ausgeliefert wird nur eigenes TypeScript | 0 Meldungen |
| `apps/web` | `svelte`, `@tauri-apps/api` | 0 Meldungen |

Dass der Worker null Laufzeit-Abhängigkeiten hat, ist kein Zufall, sondern das Ergebnis
der Entscheidung, das Protokoll selbst zu schreiben statt ein Framework zu nehmen. Es ist
auch der Grund, warum ein Wechsel weg von Cloudflare ein Nachmittag wäre.

## Geduldete Meldungen im Werkzeugbaum

### `sharp` < 0.35.4 — libheif (GHSA-g89c-p67h-r497, GHSA-2jg2-4ch7-h545)

*Gesehen 2026-09-13, `services/sync-worker`, vier Meldungen der Stufe »high«.*

Kette: `wrangler` und `@cloudflare/vitest-pool-workers` → `miniflare` → `sharp` →
`libheif`.

**Warum geduldet:** `miniflare` ist Cloudflares lokaler Emulator. Es bringt `sharp` mit,
um die Images-Bindung nachzubilden — die dieses Projekt nicht benutzt. Die Lücke steckt im
Dekodieren von HEIF-Bildern; es gibt hier keinen Weg, auf dem ein solches Bild in `sharp`
gelangt. Nichts davon wird ausgeliefert: der Worker-Bau enthält ausschließlich unseren
eigenen Code, alle Importe in `src/` sind relativ.

**Warum nicht »behoben«:** `npm audit fix --force` stuft `@cloudflare/vitest-pool-workers`
von 0.22.0 auf **0.8.30** zurück. Das bricht die Testeinrichtung und tauscht eine
Werkzeug-Meldung gegen einen realen Schaden an der Prüfbarkeit. Die Meldung verschwindet
von selbst, sobald Cloudflare `miniflare` auf `sharp` ≥ 0.35.4 hebt.

**Wann neu zu bewerten:** wenn der Worker eine Images-Bindung bekommt, oder wenn dieselbe
Meldung in einer Laufzeit-Abhängigkeit auftaucht.

## Wenn du eine neue Meldung siehst

1. `npm audit --omit=dev --audit-level=high` — schlägt sie hier an, ist sie echt und
   gehört behoben, nicht dokumentiert.
2. Schlägt sie nur im vollen Baum an: `npm ls <paket>` zeigt, wer sie hereinzieht.
   Ist es ein Werkzeug, gehört sie hierher — mit Datum, Kette, Begründung und der
   Bedingung, unter der sie neu zu bewerten ist.
3. `npm audit fix --force` niemals blind. Es stuft zurück, und ein Rückschritt an einem
   Werkzeug kann teurer sein als die Meldung.

Dieselbe Regel wie bei `deny.toml`: **eine Ausnahme ohne Begründung und Datum ist keine
Ausnahme, sondern eine Nachlässigkeit mit Konfigurationsdatei.**
