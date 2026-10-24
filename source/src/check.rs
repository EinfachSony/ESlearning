//! Antwortprüfung: tolerant bei Groß-/Kleinschreibung, Satzzeichen und Leerzeichen,
//! fehlende Akzente werden als „fast richtig“ erkannt.

use serde::Serialize;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Correct,
    /// Richtig bis auf Akzente / ñ.
    Accent,
    Wrong,
}

pub fn normalize(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| match c {
            '¿' | '?' | '¡' | '!' | '.' | ',' | ';' | ':' | '"' | '«' | '»' | '“' | '”' | '–' | '-' => ' ',
            '’' => '\'',
            c => c,
        })
        .collect::<String>()
        .to_lowercase();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn strip_accents(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' => 'a',
            'é' | 'è' => 'e',
            'í' | 'ì' => 'i',
            'ó' | 'ò' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

pub fn check(given: &str, answers: &[String]) -> Verdict {
    let g = normalize(given);
    if answers.iter().any(|a| normalize(a) == g) {
        return Verdict::Correct;
    }
    let g = strip_accents(&g);
    if answers.iter().any(|a| strip_accents(&normalize(a)) == g) {
        return Verdict::Accent;
    }
    Verdict::Wrong
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn tolerant_matching() {
        assert_eq!(check("  Hablo  alemán. ", &a(&["Hablo alemán"])), Verdict::Correct);
        assert_eq!(check("hablo aleman", &a(&["Hablo alemán"])), Verdict::Accent);
        assert_eq!(check("cómo te llamas", &a(&["¿Cómo te llamas?"])), Verdict::Correct);
        assert_eq!(check("habla", &a(&["hablo"])), Verdict::Wrong);
        assert_eq!(check("anos", &a(&["años"])), Verdict::Accent);
    }
}
