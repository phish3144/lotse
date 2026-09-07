import { beforeEach, describe, expect, it } from 'vitest';
import { createMockProvider, POSTKORB_PROJEKT_ID } from './mock';
import type { DataProvider } from './provider';

// Der Mock steht stellvertretend für den Vertrag, den auch der Tauri-Provider erfüllt:
// Was hier zugesichert wird, muss die Oberfläche von beiden erwarten dürfen.
describe('DataProvider (Mock)', () => {
  let provider: DataProvider;

  beforeEach(() => {
    provider = createMockProvider();
  });

  it('legt ein Projekt mit dem Erwartungsintervall der Vorlage an', async () => {
    const projekt = await provider.createProject('Gartenhaus', 'haus_garten', 'Fundament bis Oktober.');
    expect(projekt.titel).toBe('Gartenhaus');
    expect(projekt.status).toBe('idee');
    expect(projekt.kurs).toBe('Fundament bis Oktober.');
    expect(projekt.erwartungsintervall_tage).toBe(60);
    expect(await provider.getProject(projekt.id)).toMatchObject({ titel: 'Gartenhaus' });
  });

  it('schreibt einen geänderten Projektkopf zurück', async () => {
    const projekt = await provider.createProject('Balkon', 'generisch');
    await provider.saveProject({ ...projekt, kurs: 'Kräuter ziehen.', tags: ['garten'], erwartungsintervall_tage: 45 });
    const geladen = await provider.getProject(projekt.id);
    expect(geladen?.kurs).toBe('Kräuter ziehen.');
    expect(geladen?.tags).toEqual(['garten']);
    expect(geladen?.erwartungsintervall_tage).toBe(45);
  });

  it('liefert den Postkorb und legt ihn nur einmal an', async () => {
    const erst = await provider.postkorb();
    const zweit = await provider.postkorb();
    expect(erst.id).toBe(POSTKORB_PROJEKT_ID);
    expect(zweit.id).toBe(erst.id);
    const projekte = await provider.listProjects();
    expect(projekte.filter((p) => p.id === POSTKORB_PROJEKT_ID)).toHaveLength(1);
  });

  it('hakt einen offenen Faden ab, ohne ihn aus dem Logbuch zu entfernen', async () => {
    const projekt = await provider.createProject('Werkbank', 'hardware_maker');
    const faden = await provider.addNote(projekt.id, { quelle: 'mensch', art: 'offen', text: 'Welche Schrauben?' });

    expect((await provider.listOpenThreads()).some((n) => n.id === faden.id)).toBe(true);

    await provider.completeThread(faden.id);

    expect((await provider.listOpenThreads()).some((n) => n.id === faden.id)).toBe(false);
    const notizen = await provider.listNotes(projekt.id);
    const erledigt = notizen.find((n) => n.id === faden.id);
    expect(erledigt).toBeDefined();
    expect(erledigt?.erledigt_am).toBeTruthy();
  });

  it('erzwingt eine Übergabenotiz beim Wechsel auf pausiert', async () => {
    const projekt = await provider.createProject('Dachboden', 'haus_garten');
    await expect(provider.setStatus(projekt.id, 'pausiert')).rejects.toThrow(/Übergabenotiz/);
    const nachher = await provider.setStatus(projekt.id, 'pausiert', 'Warte auf Material.');
    expect(nachher.status).toBe('pausiert');
    const uebergaben = (await provider.listNotes(projekt.id)).filter((n) => n.art === 'uebergabe');
    expect(uebergaben.map((n) => n.text)).toContain('Warte auf Material.');
  });

  it('legt Referenzen an; nicht prüfbare Ziele sind ein gültiger Zustand', async () => {
    const projekt = await provider.createProject('Keller', 'haus_garten');
    const referenz = await provider.addReference(projekt.id, 'physisch', 'Keller, Regal 3, blaue Kiste', 'material');
    expect(referenz.pruefstatus).toBe('nicht_pruefbar');
    expect(await provider.listReferences(projekt.id)).toHaveLength(1);
    expect(await provider.checkReference(referenz.id)).toBe('nicht_pruefbar');
  });

  it('gibt Tresor-Werte nur einzeln heraus und vernichtet sie beim Löschen', async () => {
    const projekt = await provider.createProject('Fritzbox', 'software');
    const eintrag = await provider.addVaultEntry('Router', [projekt.id], 'nur_desktop', [
      { name: 'Passwort', wert: 'geheim-123' },
    ]);

    // In der Liste bleibt der Wert verdeckt.
    const liste = await provider.listVaultEntries(projekt.id);
    expect(liste).toHaveLength(1);
    expect(liste[0].felder[0].wert_verschluesselt).not.toContain('geheim');

    expect(await provider.readVaultField(eintrag.id, 'Passwort')).toBe('geheim-123');

    await provider.deleteVaultEntry(eintrag.id);
    expect(await provider.listVaultEntries(projekt.id)).toHaveLength(0);
    await expect(provider.readVaultField(eintrag.id, 'Passwort')).rejects.toThrow();
  });

  it('ordnet eine Notiz einem anderen Projekt zu, ohne den Text zu ändern', async () => {
    const postkorb = await provider.postkorb();
    const ziel = await provider.createProject('Heizung', 'hardware_maker');
    const notiz = await provider.addNote(postkorb.id, {
      quelle: 'mensch',
      art: 'log',
      text: 'Vorlauftemperatur prüfen.',
    });

    const verschoben = await provider.moveNote(notiz.id, ziel.id);
    expect(verschoben.projekt_id).toBe(ziel.id);
    expect(verschoben.text).toBe('Vorlauftemperatur prüfen.');

    expect((await provider.listNotes(postkorb.id)).some((n) => n.id === notiz.id)).toBe(false);
    expect((await provider.listNotes(ziel.id)).some((n) => n.id === notiz.id)).toBe(true);

    await expect(provider.moveNote(notiz.id, 'p_gibtsnicht')).rejects.toThrow(/Unbekanntes Projekt/);
  });

  it('löscht ein Projekt samt Notizen und Referenzen', async () => {
    const projekt = await provider.createProject('Wegwerf', 'generisch');
    await provider.addNote(projekt.id, { quelle: 'mensch', art: 'log', text: 'Eine Zeile.' });
    await provider.addReference(projekt.id, 'url', 'https://example.invalid', 'doku');

    await provider.deleteProject(projekt.id);

    expect(await provider.getProject(projekt.id)).toBeUndefined();
    expect(await provider.listNotes(projekt.id)).toHaveLength(0);
    expect(await provider.listReferences(projekt.id)).toHaveLength(0);
    expect((await provider.listProjects()).some((p) => p.id === projekt.id)).toBe(false);
  });

  it('verwirft einen Kandidaten dauerhaft', async () => {
    const vorher = await provider.listCandidates();
    expect(vorher.length).toBeGreaterThan(0);
    await provider.rejectCandidate(vorher[0].id);
    const nachher = await provider.listCandidates();
    expect(nachher.some((k) => k.id === vorher[0].id)).toBe(false);
    expect(nachher).toHaveLength(vorher.length - 1);
  });

  it('macht aus einem Kandidaten ein Projekt mit Herkunftsnotiz', async () => {
    const [kandidat] = await provider.listCandidates();
    const projekt = await provider.confirmCandidate(kandidat.id);
    expect(projekt.titel).toBe(kandidat.titel_vorschlag);
    expect(projekt.vorlage).toBe(kandidat.vorlage_vorschlag);
    const notizen = await provider.listNotes(projekt.id);
    expect(notizen.some((n) => n.quelle === 'import' && n.text.includes(kandidat.pfad))).toBe(true);
    expect((await provider.listCandidates()).some((k) => k.id === kandidat.id)).toBe(false);
  });
});
