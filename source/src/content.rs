//! Lerninhalte: Themen mit Regelerklärung und Übungen, geladen aus JSON.

use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

const EMBEDDED: &str = include_str!("../content/a1.json");

#[derive(Deserialize, Clone)]
pub struct Content {
    pub level: String,
    pub topics: Vec<Topic>,
}

#[derive(Deserialize, Clone)]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub title_es: String,
    pub summary: String,
    pub explanation: String,
    /// Ausführliche Regel: Verwendung, Beispiele, typische Fehler, Merkhilfen.
    #[serde(default)]
    pub details: String,
    pub items: Vec<Item>,
}

#[derive(Deserialize, Clone)]
pub struct Item {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub prompt: String,
    #[serde(default)]
    pub hint: Option<String>,
    pub answers: Vec<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub de: Option<String>,
    #[serde(default)]
    pub explain: Option<String>,
    /// Begründung, warum eine bestimmte falsche Antwort falsch ist (Antwort → Erklärung).
    #[serde(default)]
    pub why: BTreeMap<String, String>,
}

#[derive(Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Lücke im Satz, freie Eingabe.
    Cloze,
    /// Auswahl aus Optionen.
    Choice,
    /// Wörter in die richtige Reihenfolge bringen.
    Order,
    /// Deutschen Satz übersetzen.
    Translate,
}

impl Content {
    /// Lädt `content/a1.json` aus dem Arbeitsverzeichnis (damit man Inhalte ohne
    /// Neukompilieren ergänzen kann) und fällt sonst auf die eingebettete Version zurück.
    pub fn load() -> Result<Self, String> {
        let (src, origin) = match std::fs::read_to_string("content/a1.json") {
            Ok(s) => (s, "content/a1.json"),
            Err(_) => (EMBEDDED.to_string(), "eingebettet"),
        };
        let content: Content =
            serde_json::from_str(&src).map_err(|e| format!("Inhalte ({origin}) fehlerhaft: {e}"))?;
        content.validate()?;
        let n: usize = content.topics.iter().map(|t| t.items.len()).sum();
        println!("Inhalte geladen ({origin}): {} Themen, {n} Übungen", content.topics.len());
        Ok(content)
    }

    fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for t in &self.topics {
            for it in &t.items {
                if !ids.insert(&it.id) {
                    return Err(format!("Doppelte Übungs-ID: {}", it.id));
                }
                if it.answers.is_empty() {
                    return Err(format!("{}: keine Antwort angegeben", it.id));
                }
                if it.kind == Kind::Choice && !it.options.contains(&it.answers[0]) {
                    return Err(format!("{}: Antwort fehlt in den Optionen", it.id));
                }
                if let Some(o) = it.options.iter().find(|o| !it.answers.contains(o) && !it.why.contains_key(*o)) {
                    return Err(format!("{}: Begründung (why) für falsche Option „{o}“ fehlt", it.id));
                }
                if let Some(k) = it.why.keys().find(|k| it.answers.contains(k)) {
                    return Err(format!("{}: why-Eintrag „{k}“ ist eine richtige Antwort", it.id));
                }
            }
        }
        Ok(())
    }

    pub fn find(&self, item_id: &str) -> Option<(&Topic, &Item)> {
        self.topics
            .iter()
            .find_map(|t| t.items.iter().find(|i| i.id == item_id).map(|i| (t, i)))
    }
}
