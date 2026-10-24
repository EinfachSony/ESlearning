//! Lernfortschritt & Wiederholungsplanung (Leitner-System mit wachsenden Abständen).
//!
//! Jede Übung liegt in einer „Box“ 0–7. Richtig → nächste Box, Wiederholung später.
//! Falsch → zurück in eine niedrige Box, Wiederholung bald.

use crate::check::Verdict;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MIN: i64 = 60;
const DAY: i64 = 24 * 60 * MIN;

/// Abstand bis zur nächsten Wiederholung, indiziert nach Box.
const INTERVALS: [i64; 8] = [MIN, 10 * MIN, DAY, 3 * DAY, 7 * DAY, 16 * DAY, 35 * DAY, 90 * DAY];
pub const MAX_BOX: u8 = 7;
/// Ab dieser Box gilt eine Übung als „sitzt“.
pub const MASTERED_BOX: u8 = 4;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ItemState {
    #[serde(rename = "box")]
    pub level: u8,
    pub due: i64,
    pub seen: u32,
    pub correct: u32,
    pub lapses: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Progress {
    #[serde(default)]
    pub items: BTreeMap<String, ItemState>,
    #[serde(default)]
    pub xp: u64,
    /// XP pro Kalendertag (YYYY-MM-DD).
    #[serde(default)]
    pub daily: BTreeMap<String, u64>,
    #[serde(default = "default_goal")]
    pub daily_goal: u64,
    #[serde(default)]
    pub sprint_best: u32,
    #[serde(skip)]
    path: PathBuf,
}

fn default_goal() -> u64 {
    150
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn day_key(offset_days: i64) -> String {
    (chrono::Local::now() - chrono::Duration::days(offset_days))
        .format("%Y-%m-%d")
        .to_string()
}

pub struct Outcome {
    pub level: u8,
    pub xp_gained: u64,
}

impl Progress {
    pub fn load(path: &Path) -> Self {
        let mut p = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Progress>(&s).ok())
            .unwrap_or_else(|| Progress {
                items: BTreeMap::new(),
                xp: 0,
                daily: BTreeMap::new(),
                daily_goal: default_goal(),
                sprint_best: 0,
                path: PathBuf::new(),
            });
        p.path = path.to_path_buf();
        p
    }

    /// Atomar speichern (erst temporäre Datei, dann umbenennen).
    pub fn save(&self) {
        let tmp = self.path.with_extension("json.tmp");
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&tmp, json).and_then(|_| std::fs::rename(&tmp, &self.path)) {
                    eprintln!("Fortschritt konnte nicht gespeichert werden: {e}");
                }
            }
            Err(e) => eprintln!("Fortschritt konnte nicht serialisiert werden: {e}"),
        }
    }

    pub fn add_xp(&mut self, xp: u64) {
        self.xp += xp;
        *self.daily.entry(day_key(0)).or_default() += xp;
    }

    pub fn today_xp(&self) -> u64 {
        self.daily.get(&day_key(0)).copied().unwrap_or(0)
    }

    /// Aufeinanderfolgende Lerntage. Heute noch nicht gelernt bricht die Serie nicht.
    pub fn streak(&self) -> u32 {
        let start = if self.today_xp() > 0 { 0 } else { 1 };
        (start..)
            .take_while(|&d| self.daily.get(&day_key(d)).copied().unwrap_or(0) > 0)
            .count() as u32
    }

    /// Verarbeitet eine Antwort.
    /// `relearn`: Die Übung wurde in dieser Sitzung schon falsch beantwortet und kommt erneut.
    pub fn record(&mut self, item_id: &str, verdict: Verdict, hint_used: bool, relearn: bool, combo: u32) -> Outcome {
        let t = now();
        let st = self.items.entry(item_id.to_string()).or_default();
        st.seen += 1;

        let xp = match verdict {
            Verdict::Wrong => {
                st.lapses += 1;
                st.level = if st.level >= 3 { 2 } else { 0 };
                st.due = t + INTERVALS[0];
                0
            }
            _ if relearn || hint_used => {
                // Mit Hilfe oder nach einem Fehler richtig: noch nicht „gekonnt“ – bald wiederholen.
                st.correct += 1;
                st.due = t + INTERVALS[1];
                if relearn { 3 } else { 4 }
            }
            v => {
                st.correct += 1;
                st.level = (st.level + 1).min(MAX_BOX);
                st.due = t + INTERVALS[st.level as usize];
                let base = if v == Verdict::Correct { 10 } else { 6 };
                base + u64::from(combo.min(10))
            }
        };
        let level = st.level;
        self.add_xp(xp);
        self.save();
        Outcome { level, xp_gained: xp }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Progress {
        let mut p = Progress::load(Path::new("/nonexistent/progress.json"));
        p.path = std::env::temp_dir().join("gramatica-test-progress.json");
        p
    }

    #[test]
    fn boxes_move() {
        let mut p = fresh();
        assert_eq!(p.record("x", Verdict::Correct, false, false, 0).level, 1);
        assert_eq!(p.record("x", Verdict::Correct, false, false, 0).level, 2);
        assert_eq!(p.record("x", Verdict::Correct, false, false, 0).level, 3);
        assert_eq!(p.record("x", Verdict::Wrong, false, false, 0).level, 2);
        assert_eq!(p.record("x", Verdict::Correct, false, true, 0).level, 2);
        assert_eq!(p.record("x", Verdict::Wrong, false, false, 0).level, 0);
        assert_eq!(p.streak(), 1);
    }
}
