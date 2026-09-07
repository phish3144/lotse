import { describe, expect, it } from 'vitest';
import { datumText } from './format';

describe('datumText', () => {
  it('zeigt ein reines Datum als denselben Kalendertag', () => {
    // `new Date('2026-09-10')` wäre Mitternacht UTC; westlich davon stünde der 9. da.
    expect(datumText('2026-09-10')).toBe('10. September 2026');
  });

  it('versteht weiterhin vollständige Zeitstempel', () => {
    const iso = new Date(2026, 8, 10, 12, 0).toISOString();
    expect(datumText(iso)).toBe('10. September 2026');
  });
});
