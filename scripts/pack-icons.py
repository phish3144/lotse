"""Packt PNGs in icon.ico (Windows) und icon.icns (macOS). Beide Formate erlauben PNG-Einträge."""
import struct, sys, pathlib

src = pathlib.Path(sys.argv[1]); dst = pathlib.Path(sys.argv[2])
png = {s: (src / f"{s}.png").read_bytes() for s in (16, 32, 48, 64, 128, 256, 512, 1024)}

# ICO: Header, dann ein Verzeichniseintrag je Bild, dann die PNG-Daten.
sizes = [16, 32, 48, 64, 128, 256]
out = bytearray(struct.pack("<HHH", 0, 1, len(sizes)))
offset = 6 + 16 * len(sizes)
entries, data = bytearray(), bytearray()
for s in sizes:
    b = png[s]
    entries += struct.pack("<BBBBHHII", 0 if s == 256 else s, 0 if s == 256 else s, 0, 0, 1, 32, len(b), offset + len(data))
    data += b
(dst / "icon.ico").write_bytes(bytes(out + entries + data))

# ICNS: 'icns' + Gesamtlänge, dann Chunks (Typ, Länge inkl. Kopf, PNG).
types = {"ic07": 128, "ic08": 256, "ic09": 512, "ic10": 1024, "ic11": 32, "ic12": 64, "ic13": 256, "ic14": 512}
body = bytearray()
for t, s in types.items():
    b = png[s]
    body += t.encode("ascii") + struct.pack(">I", 8 + len(b)) + b
(dst / "icon.icns").write_bytes(b"icns" + struct.pack(">I", 8 + len(body)) + bytes(body))
print("icon.ico und icon.icns geschrieben")
