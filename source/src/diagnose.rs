//! Fehlerdiagnose: erklärt, *warum* eine konkrete Antwort falsch ist.
//!
//! Reihenfolge: handgeschriebene Begründung aus den Inhalten (`why`) → Akzente →
//! Verbformen (Person, Stammwechsel, Verbklasse, falsches Verb) → Begleiter
//! (Artikel, Possessiv, Demonstrativ, Pronomen, Fragewörter) → Angleichung von
//! Adjektiven/Plural → bei Sätzen: fehlende, überflüssige, vertauschte Wörter.

use crate::check::{normalize, strip_accents, Verdict};
use crate::content::Item;
use crate::verbs::{self, Rule, PERSONS};
use std::collections::HashMap;
use std::sync::OnceLock;

const MAX_NOTES: usize = 4;

// ---------- Verb-Index ----------

#[derive(Clone, Copy)]
struct Form {
    inf: &'static str,
    person: usize,
    /// genaue Schreibung mit Akzenten
    text: &'static str,
}

struct Index {
    /// Echte Formen, Schlüssel ohne Akzente.
    forms: HashMap<String, Vec<Form>>,
    /// Typische Fehlformen (quero, vivemos, duermimos …) mit Erklärung.
    pseudo: HashMap<String, (Form, String)>,
    /// Infinitiv → Formen
    verbs: HashMap<&'static str, [String; 6]>,
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(build_index)
}

fn build_index() -> Index {
    let mut forms: HashMap<String, Vec<Form>> = HashMap::new();
    let mut verbs_map = HashMap::new();
    let lexicon = verbs::lexicon();
    for v in &lexicon {
        for (p, f) in v.forms.iter().enumerate() {
            let text: &'static str = Box::leak(f.clone().into_boxed_str());
            forms.entry(strip_accents(f)).or_default().push(Form { inf: v.inf, person: p, text });
        }
        verbs_map.insert(v.inf, v.forms.clone());
    }
    // „hay“ als Form von haber (es gibt).
    forms.entry("hay".into()).or_default().push(Form { inf: "haber", person: 2, text: "hay" });

    let mut pseudo = HashMap::new();
    let mut add = |cand: String, inf: &'static str, person: usize, msg: String| {
        let key = strip_accents(&cand);
        if !forms.contains_key(&key) {
            pseudo.entry(key).or_insert((Form { inf, person, text: "" }, msg));
        }
    };
    for v in &lexicon {
        let inf = v.inf;
        let class = &inf[inf.len() - 2..];
        let ends = verbs::endings(inf);
        if !matches!(v.rule, Rule::Irregular) {
            // Endungen der falschen Verbklasse
            for other in ["ar", "er", "ir"].into_iter().filter(|o| *o != class) {
                let wrong = verbs::endings(&format!("x{other}"));
                for p in 1..6 {
                    if let Some(st) = v.forms[p].strip_suffix(ends[p]) {
                        let msg = format!(
                            "„{inf}“ ist ein -{class}-Verb: Endungen -{}. Richtig für {} ist „{}“.",
                            ends.join(", -"),
                            PERSONS[p],
                            v.forms[p]
                        );
                        add(format!("{st}{}", wrong[p]), inf, p, msg);
                    }
                }
            }
        }
        if let Rule::Stem(from, to) | Rule::StemYo(from, to) = v.rule {
            let regular = verbs::regular(inf);
            let boot: &[usize] = if matches!(v.rule, Rule::StemYo(..)) { &[1, 2, 5] } else { &[0, 1, 2, 5] };
            for &p in boot {
                let msg = format!(
                    "„{inf}“ hat einen Stammwechsel {from} → {to} (außer bei nosotros/vosotros): {} → „{}“.",
                    PERSONS[p], v.forms[p]
                );
                add(regular[p].clone(), inf, p, msg);
            }
            let changed = verbs::changed_stem(inf, from, to);
            for p in [3, 4] {
                let msg = format!(
                    "Bei nosotros und vosotros gibt es keinen Stammwechsel: „{}“.",
                    v.forms[p]
                );
                add(format!("{changed}{}", ends[p]), inf, p, msg);
            }
            if matches!(v.rule, Rule::StemYo(..)) {
                let msg = format!("Die yo-Form von „{inf}“ ist unregelmäßig: „{}“.", v.forms[0]);
                add(format!("{changed}o"), inf, 0, msg.clone());
                add(regular[0].clone(), inf, 0, msg);
            }
        }
        if let Rule::Yo = v.rule {
            let msg = format!("Die yo-Form von „{inf}“ ist unregelmäßig: „{}“.", v.forms[0]);
            add(verbs::regular(inf)[0].clone(), inf, 0, msg);
        }
    }
    Index { forms, pseudo, verbs: verbs_map }
}

/// Formen zu einer (evtl. ohne Akzente getippten) Eingabe.
fn verb_forms(w: &str) -> &'static [Form] {
    index().forms.get(&strip_accents(w)).map_or(&[], |v| v.as_slice())
}

/// Formen mit genau dieser Schreibung (für die Musterlösung: „cómo“ ist nicht „como“).
fn exact_forms(w: &str) -> Vec<Form> {
    verb_forms(w).iter().filter(|f| f.text == w).copied().collect()
}

fn is_infinitive(w: &str) -> Option<&'static str> {
    let w = strip_accents(w);
    let base = w.strip_suffix("se").filter(|b| b.ends_with('r')).unwrap_or(&w);
    index().verbs.keys().find(|inf| **inf == base).copied().or((base == "haber").then_some("haber"))
}

/// Regeln für häufig verwechselte Verbpaare.
fn pair_rule(a: &str, b: &str) -> Option<&'static str> {
    let has = |x: &str, y: &str| (a == x && b == y) || (a == y && b == x);
    if has("ser", "estar") {
        Some("Faustregel: ser = was/wer etwas ist (Identität, Herkunft, Beruf, Eigenschaft); estar = wo und wie etwas gerade ist (Ort, Zustand, Befinden).")
    } else if has("haber", "estar") {
        Some("hay = es gibt, für Unbestimmtes (un/una, Zahlen, Nomen ohne Artikel); estar = sich befinden, für Bestimmtes (el/la, mi …, Namen).")
    } else if b == "tener" && (a == "ser" || a == "estar") {
        Some("Alter, Hunger, Durst, Kälte, Wärme und Müdigkeit „hat“ man im Spanischen: tener años / hambre / sed / frío / calor / sueño.")
    } else if has("saber", "conocer") {
        Some("saber = etwas wissen (Fakten); conocer = jemanden/etwas kennen (Personen, Orte).")
    } else {
        None
    }
}

fn verb_note(given: &str, expected: &str) -> Option<String> {
    let exp_inf = is_infinitive(expected);
    let g_forms = verb_forms(given);

    // Gesucht ist der Infinitiv
    if let Some(inf) = exp_inf.filter(|_| exact_forms(expected).is_empty()) {
        if g_forms.iter().any(|f| f.inf == inf) {
            return Some(format!(
                "Hier gehört der Infinitiv (Grundform) hin: „{expected}“. Nach ir a, tener que, poder, querer … wird das zweite Verb nicht konjugiert."
            ));
        }
        return None;
    }

    let e = *exact_forms(expected).first()?;
    let exp_person = PERSONS[e.person];

    // Infinitiv statt konjugierter Form
    if is_infinitive(given) == Some(e.inf) {
        return Some(format!(
            "„{given}“ ist der Infinitiv (Grundform). Das Verb muss zur Person passen: {exp_person} → „{expected}“."
        ));
    }
    // Typische Fehlform
    if let Some((f, msg)) = index().pseudo.get(&strip_accents(given)) {
        if f.inf == e.inf {
            let mut msg = msg.clone();
            if f.person != e.person {
                msg.push_str(&format!(" Gesucht ist aber die Form für {exp_person}: „{expected}“."));
            }
            return Some(msg);
        }
    }
    // Gleiches Verb, andere Person
    if let Some(g) = g_forms.iter().find(|f| f.inf == e.inf) {
        if matches!(e.inf, "gustar" | "encantar") {
            return Some(if g.person == 0 {
                format!("„{given}“ hieße „ich gefalle“. Bei {} gefällt die Sache: Singular oder Infinitiv → gusta, Plural → gustan.", e.inf)
            } else {
                format!(
                    "{} richtet sich nach dem, was gefällt, nicht nach der Person: Singular oder Infinitiv → „{}“, Plural → „{}“.",
                    e.inf,
                    index().verbs[e.inf][2],
                    index().verbs[e.inf][5]
                )
            });
        }
        return Some(format!(
            "„{given}“ ist die Form für {}. Gesucht ist die Form für {exp_person}: „{expected}“.",
            PERSONS[g.person]
        ));
    }
    // Anderes Verb
    if let Some(g) = g_forms.first() {
        let name = |inf: &str| if inf == "haber" { "hay (es gibt)".to_string() } else { inf.to_string() };
        let mut msg = format!(
            "„{given}“ ist eine Form von {}, hier braucht es {}: „{expected}“.",
            name(g.inf),
            name(e.inf)
        );
        if let Some(rule) = pair_rule(g.inf, e.inf) {
            msg.push(' ');
            msg.push_str(rule);
        }
        return Some(msg);
    }
    // Fehlform eines anderen Verbs (z. B. „quero“, gesucht „puedo“)
    let (f, _) = index().pseudo.get(&strip_accents(given))?;
    Some(format!(
        "„{given}“ ist keine korrekte Form (gemeint war wohl {} von {}: „{}“). Gesucht ist hier {} von {}: „{expected}“.",
        PERSONS[f.person],
        f.inf,
        index().verbs[f.inf][f.person],
        exp_person,
        e.inf
    ))
}

// ---------- Begleiter & Funktionswörter ----------

struct Word {
    w: &'static str,
    cat: &'static str,
    gender: char,
    number: char,
    desc: &'static str,
}

const fn wd(w: &'static str, cat: &'static str, gender: char, number: char, desc: &'static str) -> Word {
    Word { w, cat, gender, number, desc }
}

static WORDS: &[Word] = &[
    wd("el", "art", 'm', 's', "bestimmter Artikel, maskulin Singular"),
    wd("la", "art", 'f', 's', "bestimmter Artikel, feminin Singular"),
    wd("los", "art", 'm', 'p', "bestimmter Artikel, maskulin Plural"),
    wd("las", "art", 'f', 'p', "bestimmter Artikel, feminin Plural"),
    wd("un", "art-i", 'm', 's', "unbestimmter Artikel, maskulin Singular"),
    wd("una", "art-i", 'f', 's', "unbestimmter Artikel, feminin Singular"),
    wd("unos", "art-i", 'm', 'p', "unbestimmter Artikel (einige), maskulin Plural"),
    wd("unas", "art-i", 'f', 'p', "unbestimmter Artikel (einige), feminin Plural"),
    wd("este", "dem1", 'm', 's', "dieser (hier), maskulin Singular"),
    wd("esta", "dem1", 'f', 's', "diese (hier), feminin Singular"),
    wd("estos", "dem1", 'm', 'p', "diese (hier), maskulin Plural"),
    wd("estas", "dem1", 'f', 'p', "diese (hier), feminin Plural"),
    wd("esto", "dem1", 'n', 's', "das hier, neutral (steht allein, nie vor einem Nomen)"),
    wd("ese", "dem2", 'm', 's', "der da, maskulin Singular"),
    wd("esa", "dem2", 'f', 's', "die da, feminin Singular"),
    wd("esos", "dem2", 'm', 'p', "die da, maskulin Plural"),
    wd("esas", "dem2", 'f', 'p', "die da, feminin Plural"),
    wd("eso", "dem2", 'n', 's', "das da, neutral (steht allein, nie vor einem Nomen)"),
    wd("aquel", "dem3", 'm', 's', "jener (dort), maskulin Singular"),
    wd("aquella", "dem3", 'f', 's', "jene (dort), feminin Singular"),
    wd("aquellos", "dem3", 'm', 'p', "jene (dort), maskulin Plural"),
    wd("aquellas", "dem3", 'f', 'p', "jene (dort), feminin Plural"),
    wd("mi", "pos1", '-', 's', "mein (ein Ding)"),
    wd("mis", "pos1", '-', 'p', "mein (mehrere Dinge)"),
    wd("tu", "pos2", '-', 's', "dein (ein Ding)"),
    wd("tus", "pos2", '-', 'p', "dein (mehrere Dinge)"),
    wd("su", "pos3", '-', 's', "sein/ihr/Ihr (ein Ding)"),
    wd("sus", "pos3", '-', 'p', "sein/ihr/Ihr (mehrere Dinge)"),
    wd("nuestro", "pos4", 'm', 's', "unser, maskulin Singular"),
    wd("nuestra", "pos4", 'f', 's', "unser, feminin Singular"),
    wd("nuestros", "pos4", 'm', 'p', "unser, maskulin Plural"),
    wd("nuestras", "pos4", 'f', 'p', "unser, feminin Plural"),
    wd("vuestro", "pos5", 'm', 's', "euer, maskulin Singular"),
    wd("vuestra", "pos5", 'f', 's', "euer, feminin Singular"),
    wd("vuestros", "pos5", 'm', 'p', "euer, maskulin Plural"),
    wd("vuestras", "pos5", 'f', 'p', "euer, feminin Plural"),
    wd("me", "pr1", '-', 's', "mir/mich (zu yo)"),
    wd("te", "pr2", '-', 's', "dir/dich (zu tú)"),
    wd("se", "pr3", '-', '-', "sich (zu él, ella, usted, ellos, ustedes)"),
    wd("nos", "pr4", '-', 'p', "uns (zu nosotros)"),
    wd("os", "pr5", '-', 'p', "euch (zu vosotros)"),
    wd("le", "pr3i", '-', 's', "ihm/ihr/Ihnen (indirektes Objekt, eine Person)"),
    wd("les", "pr3i", '-', 'p', "ihnen/Ihnen (indirektes Objekt, mehrere Personen)"),
    wd("yo", "sub1", '-', 's', "ich"),
    wd("tú", "sub2", '-', 's', "du"),
    wd("él", "sub3", 'm', 's', "er"),
    wd("ella", "sub3", 'f', 's', "sie (Singular)"),
    wd("usted", "sub3", '-', 's', "Sie (höflich, eine Person)"),
    wd("nosotros", "sub4", 'm', 'p', "wir (gemischt/männlich)"),
    wd("nosotras", "sub4", 'f', 'p', "wir (nur Frauen)"),
    wd("vosotros", "sub5", 'm', 'p', "ihr (gemischt/männlich)"),
    wd("vosotras", "sub5", 'f', 'p', "ihr (nur Frauen)"),
    wd("ellos", "sub6", 'm', 'p', "sie (Plural, gemischt/männlich)"),
    wd("ellas", "sub6", 'f', 'p', "sie (Plural, nur Frauen)"),
    wd("ustedes", "sub6", '-', 'p', "Sie (höflich, mehrere Personen)"),
    wd("qué", "q", '-', '-', "was"),
    wd("quién", "q", '-', 's', "wer (eine Person)"),
    wd("quiénes", "q", '-', 'p', "wer (mehrere Personen)"),
    wd("dónde", "q", '-', '-', "wo (Ort)"),
    wd("adónde", "q", '-', '-', "wohin (Richtung)"),
    wd("cuándo", "q", '-', '-', "wann"),
    wd("cómo", "q", '-', '-', "wie"),
    wd("cuál", "q", '-', 's', "welche(r) – Auswahl"),
    wd("cuánto", "qn", 'm', 's', "wie viel, maskulin Singular"),
    wd("cuánta", "qn", 'f', 's', "wie viel, feminin Singular"),
    wd("cuántos", "qn", 'm', 'p', "wie viele, maskulin Plural"),
    wd("cuántas", "qn", 'f', 'p', "wie viele, feminin Plural"),
    wd("porque", "conj", '-', '-', "weil (Antwort, keine Frage)"),
    wd("que", "conj", '-', '-', "dass / Bindewort (tener que + Infinitiv = müssen)"),
    wd("a", "prep", '-', '-', "nach/zu (Richtung); auch persönliches a"),
    wd("en", "prep", '-', '-', "in/an/auf (Ort); mit (Verkehrsmittel)"),
    wd("de", "prep", '-', '-', "von/aus (Herkunft, Besitz)"),
    wd("al", "prep", 'm', 's', "a + el"),
    wd("del", "prep", 'm', 's', "de + el"),
    wd("con", "prep", '-', '-', "mit"),
    wd("conmigo", "prep", '-', '-', "mit mir"),
    wd("contigo", "prep", '-', '-', "mit dir"),
    wd("por", "prep", '-', '-', "durch/wegen/für"),
    wd("para", "prep", '-', '-', "für/um zu"),
    wd("no", "neg", '-', '-', "nicht"),
    wd("nada", "neg", '-', '-', "nichts"),
    wd("nadie", "neg", '-', '-', "niemand"),
    wd("algo", "neg", '-', '-', "etwas"),
    wd("muy", "adv", '-', '-', "sehr"),
    wd("uno", "num", 'm', 's', "eins / einer (steht allein; vor einem Nomen wird es zu un)"),
];

fn word(w: &str) -> Option<&'static Word> {
    WORDS.iter().find(|x| x.w == w).or_else(|| {
        // „donde“ (ohne Akzent getippt) → dónde; aber „está“ ist nicht „esta“
        (strip_accents(w) == w).then(|| WORDS.iter().find(|x| strip_accents(x.w) == w)).flatten()
    })
}

fn family(cat: &str) -> &str {
    cat.trim_end_matches(|c: char| c.is_ascii_digit() || c == 'i').trim_end_matches('-')
}

fn word_note(given: &str, expected: &str) -> Option<String> {
    let (g, e) = (word(given)?, word(expected));
    let Some(e) = e else {
        return Some(format!("„{given}“ = {}. Gesucht ist „{expected}“.", g.desc));
    };
    let mut diff = Vec::new();
    if family(g.cat) == family(e.cat) && g.cat != e.cat {
        diff.push(match family(e.cat) {
            "dem" => "Entfernung: hier (este) / da (ese) / dort (aquel)",
            "art" => "bestimmt (el/la) oder unbestimmt (un/una)",
            "pos" | "pr" | "sub" => "Person",
            _ => "Bedeutung",
        });
    }
    if g.gender != e.gender && !matches!(g.gender, '-' | 'n') && !matches!(e.gender, '-' | 'n') {
        diff.push("Genus (maskulin/feminin)");
    }
    if g.number != e.number && g.number != '-' && e.number != '-' {
        diff.push("Zahl (Singular/Plural)");
    }
    if e.gender == 'n' && g.gender != 'n' {
        diff.push("neutral: ohne Nomen, für eine unbekannte Sache");
    }
    let tail = if diff.is_empty() { String::new() } else { format!(" Es passt nicht: {}.", diff.join(", ")) };
    Some(format!("„{given}“ = {}. Gesucht ist „{expected}“ = {}.{tail}", g.desc, e.desc))
}

// ---------- Angleichung (Adjektive, Plural) ----------

fn ending(w: &str) -> Option<(&str, char, char)> {
    for (suf, g, n) in [("os", 'm', 'p'), ("as", 'f', 'p'), ("es", '-', 'p'), ("o", 'm', 's'), ("a", 'f', 's'), ("e", '-', 's')] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.chars().count() >= 2 {
                return Some((stem, g, n));
            }
        }
    }
    None
}

fn features(g: char, n: char) -> String {
    let gs = match g {
        'm' => "maskulin ",
        'f' => "feminin ",
        _ => "",
    };
    format!("{gs}{}", if n == 'p' { "Plural" } else { "Singular" })
}

fn agreement_note(given: &str, expected: &str) -> Option<String> {
    if !exact_forms(expected).is_empty() || word(expected).is_some() {
        return None; // Verben und Funktionswörter haben eigene Erklärungen
    }
    let (g, e) = (strip_accents(given), strip_accents(expected));
    if e.ends_with("ces") && (g == format!("{}zes", &e[..e.len() - 3]) || g == format!("{}zs", &e[..e.len() - 3])) {
        return Some(format!("Wörter auf -z bilden den Plural mit -ces: „{expected}“."));
    }
    if e == format!("{g}es") {
        return Some(format!("Endet ein Wort auf Konsonant, kommt im Plural -es dazu: „{expected}“."));
    }
    if e.ends_with("es") && g == format!("{}s", &e[..e.len() - 2]) {
        return Some(format!("Nach Konsonant lautet die Pluralendung -es, nicht -s: „{expected}“."));
    }
    if e == format!("{g}s") {
        return Some(format!("Hier ist Plural nötig: „{expected}“."));
    }
    if g == format!("{e}s") || g == format!("{e}es") {
        return Some(format!("„{given}“ ist Plural, gesucht ist Singular: „{expected}“."));
    }
    if e == format!("{g}a") || e == format!("{g}as") {
        return Some(format!("Adjektive auf Konsonant (z. B. Nationalitäten) bilden die feminine Form mit -a: „{expected}“."));
    }
    let ((gs, gg, gn), (es, eg, en)) = (ending(&g)?, ending(&e)?);
    if gs != es || (gg == eg && gn == en) {
        return None;
    }
    if es.len() >= 2 && (gg == 'f' || gg == 'm') && eg == '-' {
        return Some(format!(
            "Wörter auf -e haben keine Form auf -o/-a, nur Singular (-e) und Plural (-es): „{expected}“."
        ));
    }
    Some(format!(
        "„{given}“ ist {}, hier muss es {} sein: „{expected}“. Adjektive richten sich in Genus und Zahl nach dem Nomen.",
        features(gg, gn),
        features(eg, en)
    ))
}

// ---------- Akzente ----------

/// Wortpaare, bei denen der Akzent die Bedeutung ändert: (mit, ohne, Bedeutung mit, Bedeutung ohne).
const ACCENT_PAIRS: &[(&str, &str, &str, &str)] = &[
    ("está", "esta", "ist (estar)", "diese"),
    ("estás", "estas", "bist (estar)", "diese (Plural)"),
    ("él", "el", "er", "der (Artikel)"),
    ("tú", "tu", "du", "dein"),
    ("mí", "mi", "mir/mich (nach Präposition)", "mein"),
    ("sé", "se", "ich weiß (saber)", "sich"),
    ("sí", "si", "ja", "wenn/ob"),
    ("té", "te", "Tee", "dir/dich"),
    ("más", "mas", "mehr", "aber (veraltet)"),
];

/// Liefert (Hinweis, ändert Bedeutung?).
fn accent_note(given: &str, expected: &str) -> Option<(String, bool)> {
    if given == expected || strip_accents(given) != strip_accents(expected) {
        return None;
    }
    for (with, without, m_with, m_without) in ACCENT_PAIRS {
        if (expected == *with && given == *without) || (expected == *without && given == *with) {
            let (gm, em) = if given == *with { (m_with, m_without) } else { (m_without, m_with) };
            return Some((
                format!("„{given}“ heißt „{gm}“, „{expected}“ heißt „{em}“. Hier ändert der Akzent die Bedeutung!"),
                true,
            ));
        }
    }
    if word(expected).is_some_and(|w| w.cat == "q" || w.cat == "qn") {
        return Some((format!("Fragewörter tragen immer einen Akzent: „{expected}“."), false));
    }
    if expected.contains('ñ') && !given.contains('ñ') {
        return Some((format!("ñ und n sind verschiedene Buchstaben: „{expected}“."), false));
    }
    let missing: Vec<char> = expected
        .chars()
        .zip(given.chars())
        .filter(|(e, g)| e != g && "áéíóú".contains(*e))
        .map(|(e, _)| e)
        .collect();
    if !missing.is_empty() {
        let list: Vec<String> = missing.iter().map(|c| c.to_string()).collect();
        return Some((format!("Akzent fehlt: „{expected}“ ({}).", list.join(", ")), false));
    }
    Some((format!("Achte auf die Akzente: „{expected}“."), false))
}

// ---------- Einzelwort ----------

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

fn single_word(given: &str, expected: &str) -> Option<String> {
    if let Some((note, _)) = accent_note(given, expected) {
        return Some(note);
    }
    verb_note(given, expected)
        .or_else(|| word_note(given, expected))
        .or_else(|| agreement_note(given, expected))
        .or_else(|| {
            let d = levenshtein(&strip_accents(given), &strip_accents(expected));
            (d <= 2 && expected.chars().count() >= 4)
                .then(|| format!("Fast – vermutlich ein Tippfehler. Richtig geschrieben: „{expected}“."))
        })
        .or_else(|| {
            // Bekanntes Verb, aber nichts Passendes: wenigstens sagen, was es ist.
            verb_forms(given).first().map(|f| {
                format!("„{given}“ ist {} von {} – das passt hier nicht. Gesucht ist „{expected}“.", PERSONS[f.person], f.inf)
            })
        })
        .or_else(|| {
            // Letzter Rückfall: wenigstens sagen, was gesucht ist.
            let what = word(expected).map(|w| w.desc.to_string()).or_else(|| {
                exact_forms(expected).first().map(|f| format!("{} von {}", PERSONS[f.person], f.inf))
            })?;
            Some(format!("„{given}“ gibt es so nicht. Gesucht ist „{expected}“ = {what}."))
        })
}

// ---------- Sätze ----------

enum Op {
    Same(usize, usize),
    Extra(usize),
    Missing(usize),
}

/// Wortweiser Abgleich (längste gemeinsame Teilfolge, Vergleich ohne Akzente).
fn align(g: &[String], e: &[String]) -> Vec<Op> {
    let (gs, es): (Vec<String>, Vec<String>) = (g.iter().map(|w| strip_accents(w)).collect(), e.iter().map(|w| strip_accents(w)).collect());
    let (n, m) = (gs.len(), es.len());
    let mut l = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            l[i][j] = if gs[i] == es[j] { l[i + 1][j + 1] + 1 } else { l[i + 1][j].max(l[i][j + 1]) };
        }
    }
    let (mut i, mut j, mut ops) = (0, 0, Vec::new());
    while i < n || j < m {
        if i < n && j < m && gs[i] == es[j] {
            ops.push(Op::Same(i, j));
            i += 1;
            j += 1;
        } else if j < m && (i == n || l[i][j + 1] >= l[i + 1][j]) {
            ops.push(Op::Missing(j));
            j += 1;
        } else {
            ops.push(Op::Extra(i));
            i += 1;
        }
    }
    ops
}

fn cost(ops: &[Op]) -> usize {
    ops.iter().filter(|o| !matches!(o, Op::Same(..))).count()
}

fn words(s: &str) -> Vec<String> {
    normalize(s).split(' ').filter(|w| !w.is_empty()).map(String::from).collect()
}

/// Personalpronomen, das nicht zur folgenden Verbform passt („yo estudiamos“).
fn pronoun_mismatch(g: &[String], i: usize) -> Option<String> {
    let pron = word(&g[i]).filter(|w| w.cat.starts_with("sub"))?;
    let person = pron.cat[3..].parse::<usize>().ok()? - 1;
    let verb = g.get(i + 1)?;
    let forms = verb_forms(verb);
    let f = forms.first()?;
    (!forms.iter().any(|f| f.person == person)).then(|| {
        format!(
            "„{}“ passt nicht zur Verbform „{verb}“ – das ist die Form für {}. Zu {} gehört „{}“.",
            g[i],
            PERSONS[f.person],
            g[i],
            index().verbs.get(f.inf).map_or("?", |v| v[person].as_str())
        )
    })
}

fn sentence_notes(g: &[String], e: &[String]) -> Vec<String> {
    let stripped = |v: &[String]| v.iter().map(|w| strip_accents(w)).collect::<Vec<_>>();
    let (sg, se) = (stripped(g), stripped(e));
    let mut notes = Vec::new();

    if sg != se {
        let (mut a, mut b) = (sg.clone(), se.clone());
        a.sort();
        b.sort();
        if a == b {
            return vec![format!("Alle Wörter stimmen, nur die Reihenfolge nicht: „{}“.", e.join(" "))];
        }
    }
    let mismatched: Vec<usize> = (0..g.len()).filter(|&i| pronoun_mismatch(g, i).is_some()).collect();
    notes.extend(mismatched.iter().filter_map(|&i| pronoun_mismatch(g, i)));

    let ops = align(g, e);
    let mut k = 0;
    while k < ops.len() {
        if let Op::Same(i, j) = ops[k] {
            // gleiches Wort, evtl. Akzentfehler
            if let Some((n, _)) = accent_note(&g[i], &e[j]) {
                notes.push(n);
            }
            k += 1;
            continue;
        }
        let (mut extra, mut missing) = (Vec::new(), Vec::new());
        while k < ops.len() && !matches!(ops[k], Op::Same(..)) {
            match ops[k] {
                Op::Extra(i) if mismatched.contains(&i) => {}
                Op::Extra(i) => extra.push(&g[i]),
                Op::Missing(j) => missing.push(&e[j]),
                Op::Same(..) => {}
            }
            k += 1;
        }
        let pairs = extra.len().min(missing.len());
        for p in 0..pairs {
            notes.push(single_word(extra[p], missing[p]).unwrap_or_else(|| format!("„{}“ statt „{}“.", extra[p], missing[p])));
        }
        for w in &extra[pairs..] {
            notes.push(match word(w) {
                Some(x) => format!("„{w}“ ({}) ist hier überflüssig.", x.desc),
                None => format!("„{w}“ ist hier überflüssig."),
            });
        }
        for w in &missing[pairs..] {
            notes.push(match word(w) {
                Some(x) => format!("Es fehlt „{w}“ ({}).", x.desc),
                None => format!("Es fehlt „{w}“."),
            });
        }
    }
    notes
}

// ---------- Einstieg ----------

/// Erklärung für eine einzelne Form (Verb-Sprint): Eingabe gegen erwartete Form.
pub fn explain_word(given: &str, expected: &str) -> Vec<String> {
    let (g, e) = (normalize(given), normalize(expected));
    if g == e {
        return vec![];
    }
    single_word(&g, &e).into_iter().collect()
}

/// Analysiert eine Antwort. Kann das Urteil verschärfen (Akzent, der die Bedeutung ändert → falsch).
pub fn analyze(item: &Item, given: &str, verdict: Verdict) -> (Verdict, Vec<String>) {
    if verdict == Verdict::Correct {
        return (verdict, vec![]);
    }
    // Handgeschriebene Begründung
    let g_norm = normalize(given);
    if let Some((_, why)) = item.why.iter().find(|(k, _)| normalize(k) == g_norm) {
        return (verdict, vec![why.clone()]);
    }

    let g = words(given);
    // Nächstliegende akzeptierte Lösung
    let e = item
        .answers
        .iter()
        .map(|a| words(a))
        .min_by_key(|a| cost(&align(&g, a)))
        .unwrap_or_default();

    // Getrennt-/Zusammenschreibung (porque / por qué)
    if g.len() != e.len() && strip_accents(&g.concat()) == strip_accents(&e.concat()) {
        let mut note = format!("Achte auf Getrennt- bzw. Zusammenschreibung: „{}“.", e.join(" "));
        if let Some(w) = g.first().filter(|_| g.len() == 1).and_then(|w| word(w)) {
            note.push_str(&format!(" „{}“ heißt {}.", g[0], w.desc));
        }
        if e.join(" ") == "por qué" {
            note.push_str(" Frage: por qué (warum), Antwort: porque (weil).");
        }
        return (Verdict::Wrong, vec![note]);
    }

    let mut verdict = verdict;
    let mut notes = if g.len() == 1 && e.len() == 1 {
        if let Some((n, changes_meaning)) = accent_note(&g[0], &e[0]) {
            if changes_meaning {
                verdict = Verdict::Wrong;
            }
            vec![n]
        } else {
            single_word(&g[0], &e[0]).into_iter().collect()
        }
    } else {
        if verdict == Verdict::Accent {
            for (gw, ew) in g.iter().zip(e.iter()) {
                if accent_note(gw, ew).is_some_and(|(_, strong)| strong) {
                    verdict = Verdict::Wrong;
                }
            }
        }
        sentence_notes(&g, &e)
    };
    notes.dedup();
    notes.truncate(MAX_NOTES);
    (verdict, notes)
}

#[cfg(test)]
mod tests_support {
    use super::*;
    use crate::content::Kind;

    pub fn item(answers: &[&str]) -> Item {
        Item {
            id: "t".into(),
            kind: Kind::Cloze,
            prompt: String::new(),
            hint: None,
            answers: answers.iter().map(|s| s.to_string()).collect(),
            options: vec![],
            de: None,
            explain: None,
            why: Default::default(),
        }
    }

    pub fn notes(answers: &[&str], given: &str) -> String {
        let v = crate::check::check(given, &item(answers).answers);
        analyze(&item(answers), given, v).1.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Kind;

    fn item(answers: &[&str]) -> Item {
        Item {
            id: "t".into(),
            kind: Kind::Cloze,
            prompt: String::new(),
            hint: None,
            answers: answers.iter().map(|s| s.to_string()).collect(),
            options: vec![],
            de: None,
            explain: None,
            why: Default::default(),
        }
    }

    fn notes(answers: &[&str], given: &str) -> (Verdict, String) {
        let v = crate::check::check(given, &item(answers).answers);
        let (v, n) = analyze(&item(answers), given, v);
        (v, n.join(" | "))
    }

    #[test]
    fn verbs() {
        assert!(notes(&["hablo"], "hablas").1.contains("Form für tú"));
        assert!(notes(&["hablo"], "hablar").1.contains("Infinitiv"));
        assert!(notes(&["quiero"], "quero").1.contains("Stammwechsel e → ie"));
        assert!(notes(&["dormimos"], "duermimos").1.contains("keinen Stammwechsel"));
        assert!(notes(&["vivimos"], "vivemos").1.contains("-ir-Verb"));
        assert!(notes(&["hago"], "haco").1.contains("yo-Form"));
        assert!(notes(&["tengo"], "teno").1.contains("yo-Form"));
        assert!(notes(&["está"], "es").1.contains("Faustregel"));
        assert!(notes(&["hay"], "está").1.contains("es gibt"));
        assert!(notes(&["gustan"], "gusta").1.contains("was gefällt"));
        assert!(notes(&["llover"], "llueve").1.contains("Infinitiv"));
    }

    #[test]
    fn words_and_agreement() {
        assert!(notes(&["Las"], "los").1.contains("Genus"));
        assert!(notes(&["Esta"], "este").1.contains("Genus"));
        assert!(notes(&["esto"], "este").1.contains("neutral"));
        assert!(notes(&["les"], "le").1.contains("Zahl"));
        assert!(notes(&["blanca"], "blanco").1.contains("feminin Singular"));
        assert!(notes(&["lápices"], "lapizes").1.contains("-ces"));
        assert!(notes(&["flores"], "flors").1.contains("-es"));
        assert!(notes(&["alemana"], "alemán").1.contains("-a"));
        assert!(notes(&["Cuántos"], "cuántas").1.contains("Genus"));
    }

    #[test]
    fn accents() {
        let (v, n) = notes(&["está"], "esta");
        assert_eq!(v, Verdict::Wrong);
        assert!(n.contains("Bedeutung"));
        let (v, n) = notes(&["estudiáis"], "estudiais");
        assert_eq!(v, Verdict::Accent);
        assert!(n.contains("á"));
        assert_eq!(notes(&["Qué"], "que").0, Verdict::Accent);
        assert!(notes(&["Por qué"], "porque").1.contains("weil"));
    }

    #[test]
    fn sentences() {
        assert!(notes(&["Soy profesor"], "Soy un profesor").1.contains("überflüssig"));
        assert!(notes(&["Tengo que limpiar la casa"], "tengo limpiar la casa").1.contains("Es fehlt „que“"));
        assert!(notes(&["Me levanto a las siete"], "a las siete me levanto").1.contains("Reihenfolge"));
        assert!(notes(&["Hablo alemán", "Yo hablo alemán"], "yo hablas alemán").1.contains("Form für tú"));
    }
}

/// Prüft alle Übungen mit automatisch erzeugten Falschantworten.
/// `cargo test --release audit -- --ignored --nocapture`
#[cfg(test)]
mod audit {
    use super::*;
    use crate::content::{Content, Kind};

    fn variants(answer: &str) -> Vec<String> {
        let mut out = Vec::new();
        let ws = words(answer);
        for (i, w) in ws.iter().enumerate() {
            let mut alts: Vec<String> = Vec::new();
            if let Some(f) = verb_forms(w).first() {
                alts.extend(index().verbs.get(f.inf).into_iter().flatten().filter(|x| *x != w).cloned());
                alts.push(f.inf.to_string());
                alts.extend(index().pseudo.iter().filter(|(_, (pf, _))| pf.inf == f.inf && pf.person == f.person).map(|(k, _)| k.clone()));
            }
            if let Some(x) = word(w) {
                alts.extend(WORDS.iter().filter(|y| family(y.cat) == family(x.cat) && y.w != x.w).map(|y| y.w.to_string()));
            }
            if let Some((stem, _, _)) = ending(w) {
                alts.extend(["o", "a", "os", "as"].iter().map(|e| format!("{stem}{e}")).filter(|v| v != w));
            }
            if strip_accents(w) != *w {
                alts.push(strip_accents(w));
            }
            for a in alts {
                let mut v = ws.clone();
                v[i] = a;
                out.push(v.join(" "));
            }
            if ws.len() > 1 {
                let mut v = ws.clone();
                v.remove(i);
                out.push(v.join(" "));
            }
        }
        if ws.len() > 2 {
            let mut v = ws.clone();
            v.swap(0, 1);
            out.push(v.join(" "));
        }
        out.sort();
        out.dedup();
        out
    }

    #[test]
    #[ignore]
    fn audit() {
        let content = Content::load().unwrap();
        let (mut total, mut empty) = (0, 0);
        for t in &content.topics {
            for it in &t.items {
                let cands: Vec<String> = if it.kind == Kind::Choice {
                    it.options.iter().filter(|o| !it.answers.contains(o)).cloned().collect()
                } else {
                    variants(&it.answers[0])
                };
                for c in cands {
                    let v = crate::check::check(&c, &it.answers);
                    if v == Verdict::Correct {
                        continue;
                    }
                    total += 1;
                    let (v, n) = analyze(it, &c, v);
                    if n.is_empty() {
                        empty += 1;
                    }
                    println!("{}\t{}\t{:?}\t{}\t{}", it.id, it.answers[0], v, c, if n.is_empty() { "∅ KEINE ERKLÄRUNG".into() } else { n.join(" | ") });
                }
            }
        }
        println!("SUMMARY: {total} Falschantworten, {empty} ohne Erklärung");
    }
}

#[cfg(test)]
mod regressions {
    use super::tests_support::notes;

    #[test]
    fn found_in_audit() {
        assert!(!notes(&["Nosotros estudiamos español"], "nosotros estudiamos espanol").contains("Reihenfolge"));
        assert!(notes(&["Nosotros estudiamos español"], "nosotros estudiamos espanol").contains("español"));
        assert!(!notes(&["¿Cómo te llamas?"], "cóma te llamas").contains("comer"));
        assert!(!notes(&["está"], "esto").contains("diese (hier)"));
        assert!(!notes(&["lápices"], "lápicos").contains("Adjektive auf -e"));
        assert!(notes(&["Nosotros estudiamos español", "Estudiamos español"], "yo estudiamos español").contains("passt nicht zur Verbform"));
        assert!(notes(&["una"], "uno").contains("vor einem Nomen"));
    }
}
