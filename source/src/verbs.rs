//! Konjugationstabellen (Präsens) für den Verb-Sprint und die Fehlerdiagnose.

use serde::Serialize;

pub const PERSONS: [&str; 6] = ["yo", "tú", "él/ella/usted", "nosotros", "vosotros", "ellos/ellas/ustedes"];

/// Nach welcher Regel ein Verb konjugiert wird (für die Fehlerdiagnose).
#[derive(Clone, Copy)]
pub enum Rule {
    Regular,
    /// Stammwechsel (von, zu) in yo, tú, él, ellos.
    Stem(char, &'static str),
    /// Nur die yo-Form ist unregelmäßig.
    Yo,
    /// Stammwechsel und unregelmäßige yo-Form (tener, venir, decir).
    StemYo(char, &'static str),
    /// Komplett unregelmäßig.
    Irregular,
}

#[derive(Serialize)]
pub struct Verb {
    pub inf: &'static str,
    pub de: &'static str,
    /// yo, tú, él, nosotros, vosotros, ellos
    pub forms: [String; 6],
    pub group: &'static str,
    #[serde(skip)]
    pub rule: Rule,
}

pub fn endings(inf: &str) -> [&'static str; 6] {
    match &inf[inf.len() - 2..] {
        "ar" => ["o", "as", "a", "amos", "áis", "an"],
        "er" => ["o", "es", "e", "emos", "éis", "en"],
        _ => ["o", "es", "e", "imos", "ís", "en"],
    }
}

pub fn stem(inf: &str) -> &str {
    &inf[..inf.len() - 2]
}

pub fn regular(inf: &str) -> [String; 6] {
    endings(inf).map(|e| format!("{}{e}", stem(inf)))
}

/// Stamm mit Stammwechsel (letztes Vorkommen des Vokals).
pub fn changed_stem(inf: &str, from: char, to: &str) -> String {
    let s = stem(inf);
    let pos = s.rfind(from).expect("Stammvokal fehlt");
    format!("{}{to}{}", &s[..pos], &s[pos + from.len_utf8()..])
}

/// Stammwechsel in allen Formen außer nosotros/vosotros.
fn stem_change(inf: &str, from: char, to: &str) -> [String; 6] {
    let changed = changed_stem(inf, from, to);
    let mut forms = regular(inf);
    for i in [0, 1, 2, 5] {
        forms[i] = format!("{changed}{}", endings(inf)[i]);
    }
    forms
}

fn with_yo(mut forms: [String; 6], yo: &str) -> [String; 6] {
    forms[0] = yo.to_string();
    forms
}

fn table(f: [&str; 6]) -> [String; 6] {
    f.map(String::from)
}

/// Verben für den Sprint.
pub fn all() -> Vec<Verb> {
    let mut v = Vec::new();
    let mut add = |inf, de, forms, group, rule| v.push(Verb { inf, de, forms, group, rule });

    for (inf, de) in [
        ("hablar", "sprechen"), ("trabajar", "arbeiten"), ("estudiar", "lernen, studieren"),
        ("escuchar", "hören"), ("comprar", "kaufen"), ("tomar", "nehmen, trinken"),
        ("bailar", "tanzen"), ("cantar", "singen"), ("viajar", "reisen"), ("cenar", "zu Abend essen"),
        ("llegar", "ankommen"), ("necesitar", "brauchen"),
        ("comer", "essen"), ("beber", "trinken"), ("leer", "lesen"), ("aprender", "lernen"),
        ("comprender", "verstehen"), ("correr", "laufen"),
        ("vivir", "wohnen, leben"), ("escribir", "schreiben"), ("abrir", "öffnen"), ("recibir", "bekommen"),
    ] {
        add(inf, de, regular(inf), "regelmäßig", Rule::Regular);
    }

    for (inf, de, from, to) in [
        ("querer", "wollen, mögen", 'e', "ie"), ("pensar", "denken", 'e', "ie"),
        ("empezar", "anfangen", 'e', "ie"), ("preferir", "bevorzugen", 'e', "ie"),
        ("cerrar", "schließen", 'e', "ie"),
        ("poder", "können", 'o', "ue"), ("dormir", "schlafen", 'o', "ue"),
        ("volver", "zurückkehren", 'o', "ue"), ("costar", "kosten", 'o', "ue"),
        ("encontrar", "finden", 'o', "ue"), ("jugar", "spielen", 'u', "ue"),
        ("pedir", "bestellen, bitten", 'e', "i"), ("repetir", "wiederholen", 'e', "i"),
        ("servir", "servieren", 'e', "i"),
    ] {
        add(inf, de, stem_change(inf, from, to), "Stammwechsel", Rule::Stem(from, to));
    }

    for (inf, de, yo) in [
        ("hacer", "machen", "hago"), ("salir", "ausgehen", "salgo"), ("poner", "stellen, legen", "pongo"),
        ("saber", "wissen", "sé"), ("conocer", "kennen", "conozco"), ("traer", "bringen", "traigo"),
        ("ver", "sehen", "veo"),
    ] {
        let mut forms = with_yo(regular(inf), yo);
        if inf == "ver" {
            forms[4] = "veis".into(); // einsilbig, ohne Akzent
        }
        add(inf, de, forms, "yo-unregelmäßig", Rule::Yo);
    }

    add("tener", "haben", with_yo(stem_change("tener", 'e', "ie"), "tengo"), "unregelmäßig", Rule::StemYo('e', "ie"));
    add("venir", "kommen", with_yo(stem_change("venir", 'e', "ie"), "vengo"), "unregelmäßig", Rule::StemYo('e', "ie"));
    add("decir", "sagen", with_yo(stem_change("decir", 'e', "i"), "digo"), "unregelmäßig", Rule::StemYo('e', "i"));
    add("ser", "sein", table(["soy", "eres", "es", "somos", "sois", "son"]), "unregelmäßig", Rule::Irregular);
    add("estar", "sein, sich befinden", table(["estoy", "estás", "está", "estamos", "estáis", "están"]), "unregelmäßig", Rule::Irregular);
    add("ir", "gehen, fahren", table(["voy", "vas", "va", "vamos", "vais", "van"]), "unregelmäßig", Rule::Irregular);
    add("dar", "geben", table(["doy", "das", "da", "damos", "dais", "dan"]), "unregelmäßig", Rule::Irregular);
    v
}

/// Alle Verben, die in den Übungen vorkommen (Sprint + weitere), für die Fehlerdiagnose.
pub fn lexicon() -> Vec<Verb> {
    let mut v = all();
    for (inf, de) in [
        ("tocar", "spielen (Instrument)"), ("buscar", "suchen"), ("limpiar", "putzen"), ("desayunar", "frühstücken"),
        ("ayudar", "helfen"), ("llamar", "rufen, heißen (llamarse)"), ("levantar", "aufstehen (levantarse)"),
        ("duchar", "duschen (ducharse)"), ("lavar", "waschen (lavarse)"), ("gustar", "gefallen"),
        ("encantar", "sehr gefallen"),
    ] {
        v.push(Verb { inf, de, forms: regular(inf), group: "regelmäßig", rule: Rule::Regular });
    }
    for (inf, de, from, to) in [("acostar", "ins Bett gehen (acostarse)", 'o', "ue"), ("vestir", "anziehen (vestirse)", 'e', "i"), ("despertar", "aufwachen (despertarse)", 'e', "ie"), ("llover", "regnen", 'o', "ue")] {
        v.push(Verb { inf, de, forms: stem_change(inf, from, to), group: "Stammwechsel", rule: Rule::Stem(from, to) });
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conjugations() {
        assert_eq!(regular("vivir")[3], "vivimos");
        assert_eq!(regular("comer")[4], "coméis");
        assert_eq!(stem_change("empezar", 'e', "ie")[2], "empieza");
        assert_eq!(stem_change("empezar", 'e', "ie")[3], "empezamos");
        assert_eq!(stem_change("encontrar", 'o', "ue")[0], "encuentro");
        assert_eq!(stem_change("repetir", 'e', "i")[5], "repiten");
        assert_eq!(stem_change("jugar", 'u', "ue")[1], "juegas");
        let tener = &all().into_iter().find(|v| v.inf == "tener").unwrap().forms;
        assert_eq!(tener.clone(), table(["tengo", "tienes", "tiene", "tenemos", "tenéis", "tienen"]));
        let decir = &all().into_iter().find(|v| v.inf == "decir").unwrap().forms;
        assert_eq!(decir.clone(), table(["digo", "dices", "dice", "decimos", "decís", "dicen"]));
        let venir = &all().into_iter().find(|v| v.inf == "venir").unwrap().forms;
        assert_eq!(venir.clone(), table(["vengo", "vienes", "viene", "venimos", "venís", "vienen"]));
    }
}
