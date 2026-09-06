//! Hybrid Logical Clock (HLC).
//!
//! Format: `<unix_ms:013>-<counter:04>-<device_id:26>`. Lexikografisch sortierbar; die
//! Sortierung entscheidet Last-Writer-Wins beim Sync (`docs/SYNC_PROTOCOL.md`, Abschnitt 3).
//! Eine HLC ist nie kleiner als jede bereits gesehene HLC, auch wenn die Systemuhr
//! zurückspringt.

use std::cmp::Ordering;

use ulid::Ulid;

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct Hlc {
    device_id: Ulid,
    last_ms: i64,
    counter: u32,
}

impl Hlc {
    pub fn new(device_id: Ulid) -> Hlc {
        Hlc {
            device_id,
            last_ms: 0,
            counter: 0,
        }
    }

    pub fn device_id(&self) -> Ulid {
        self.device_id
    }

    /// Erzeugt den nächsten Zeitstempel. `now_ms` ist die Systemzeit.
    pub fn next(&mut self, now_ms: i64) -> String {
        if now_ms > self.last_ms {
            self.last_ms = now_ms;
            self.counter = 0;
        } else {
            self.counter += 1;
            if self.counter > 9999 {
                // Mehr als 10 000 Änderungen in einer Millisekunde: Uhr künstlich vorrücken.
                self.last_ms += 1;
                self.counter = 0;
            }
        }
        format(self.last_ms, self.counter, &self.device_id)
    }

    /// Nimmt eine fremde HLC zur Kenntnis, damit eigene Zeitstempel danach größer sind.
    pub fn observe(&mut self, remote: &str) -> Result<()> {
        let (ms, counter, _) = parse(remote)?;
        match ms.cmp(&self.last_ms) {
            Ordering::Greater => {
                self.last_ms = ms;
                self.counter = counter;
            }
            Ordering::Equal => {
                self.counter = self.counter.max(counter);
            }
            Ordering::Less => {}
        }
        Ok(())
    }

    /// Stellt die Uhr aus dem größten lokal gespeicherten Zeitstempel wieder her.
    pub fn restore(&mut self, largest_seen: Option<&str>) -> Result<()> {
        if let Some(s) = largest_seen {
            self.observe(s)?;
        }
        Ok(())
    }
}

pub fn format(ms: i64, counter: u32, device_id: &Ulid) -> String {
    format!("{ms:013}-{counter:04}-{device_id}")
}

/// Zerlegt eine HLC in Millisekunden, Zähler und Geräte-ID.
pub fn parse(s: &str) -> Result<(i64, u32, Ulid)> {
    let ungueltig = || Error::Invalid(format!("HLC ungültig: {s}"));
    if s.len() != 13 + 1 + 4 + 1 + 26 {
        return Err(ungueltig());
    }
    let (ms, rest) = s.split_at(13);
    let rest = rest.strip_prefix('-').ok_or_else(ungueltig)?;
    let (counter, rest) = rest.split_at(4);
    let device = rest.strip_prefix('-').ok_or_else(ungueltig)?;
    let ms: i64 = ms.parse().map_err(|_| ungueltig())?;
    let counter: u32 = counter.parse().map_err(|_| ungueltig())?;
    let device = Ulid::from_string(device).map_err(|_| ungueltig())?;
    Ok((ms, counter, device))
}

/// Prüft, ob ein String das HLC-Format hat.
pub fn is_valid(s: &str) -> bool {
    parse(s).is_ok()
}

/// Millisekunden-Anteil einer HLC.
pub fn ms_of(s: &str) -> Option<i64> {
    parse(s).ok().map(|(ms, _, _)| ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monoton_bei_gleicher_ms() {
        let mut c = Hlc::new(Ulid::new());
        let a = c.next(1000);
        let b = c.next(1000);
        let d = c.next(999);
        assert!(a < b);
        assert!(b < d);
        assert_eq!(ms_of(&d), Some(1000));
    }

    #[test]
    fn observe_hebt_uhr_an() {
        let dev = Ulid::new();
        let mut c = Hlc::new(dev);
        let fremd = format(5000, 7, &Ulid::new());
        c.observe(&fremd).unwrap();
        let eigen = c.next(4000);
        assert!(eigen > fremd);
        assert_eq!(ms_of(&eigen), Some(5000));
    }

    #[test]
    fn parse_und_format() {
        let dev = Ulid::new();
        let s = format(1757174400123, 1, &dev);
        assert_eq!(s.len(), 45);
        let (ms, counter, d) = parse(&s).unwrap();
        assert_eq!((ms, counter, d), (1757174400123, 1, dev));
        assert!(!is_valid("kaputt"));
    }
}
