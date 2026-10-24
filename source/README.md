# Gramática – Spanisch-Grammatik A1

Lokaler Grammatiktrainer mit Rust-Webserver (axum). Er hat 17 A1-Themen, 251 Übungen und einen Verb-Sprint mit 50 Verben. Der Fortschritt wird in `progress.json` gespeichert.

## Starten

```sh
cargo run --release
# → http://127.0.0.1:4747 (falls belegt, der nächste freie Port, siehe Konsole)
```

- Fester Port: `PORT=5050 cargo run --release`
- Andere Fortschrittsdatei: `GRAMATICA_PROGRESS=~/spanisch.json cargo run --release`
- Direkteinstieg per Lesezeichen: `/#mix`, `/#sprint`, `/#topic=ser`

## Sprachausgabe (Google Text-to-Speech)

Alle Übungssätze sind mit Googles **Chirp 3 HD**-Stimmen vertont, der natürlichsten Stimmgeneration von Google. Die Dateien liegen im Repo unter [`audio/`](audio/): eine Frauenstimme (`es-ES-Chirp3-HD-Kore`) und eine Männerstimme (`es-ES-Chirp3-HD-Charon`), wählbar unten auf der Startseite. Zum Abspielen ist **kein API-Schlüssel nötig**.

Neue oder geänderte Übungen nachvertonen:

```sh
cargo run --release -- --audio   # erzeugt nur fehlende Dateien, wartet bei Googles Anfragelimit automatisch
```

Mit Schlüssel holt der Server fehlende Sätze auch während des Übens nach und legt sie in `audio/` ab. Ohne Schlüssel und ohne Datei übernimmt die Browser-Stimme. Der Schlüssel wird in dieser Reihenfolge gesucht: Umgebungsvariable `GOOGLE_API_KEY`, `.env` (Vorlage [`.env.example`](.env.example)), `TTS_Key.txt` (Schlüssel in der ersten Zeile). `.env` und `TTS_Key.txt` stehen in der `.gitignore`. Im Google-Cloud-Projekt muss die **Cloud Text-to-Speech API** aktiviert sein.

## Wie gelernt wird und warum

| Prinzip | Umsetzung | Forschungsstand |
|---|---|---|
| **Abrufübung** (*retrieval practice / testing effect*) | Fast alles wird aktiv produziert (Lücke tippen, Satz bauen, übersetzen) statt nur gelesen. Die Regelkarte ist kurz, danach wird sofort geübt. | Einer der robustesten Lerneffekte überhaupt. Freie Produktion wirkt stärker als bloßes Wiedererkennen. |
| **Verteiltes Wiederholen** (*spacing*) | Leitner-Boxen mit Abständen von 10 Min. → 1 → 3 → 7 → 16 → 35 → 90 Tagen. Fehler schicken eine Übung zurück in eine niedrige Box. | Meta-Analyse zum L2-Lernen (Kim & Webb 2022): kurzfristig gleich gut, nach 2 Wochen klar besser als geballtes Üben. Für Grammatik gibt es weniger Studien, die Tendenz ist aber dieselbe. |
| **Wiederlernen in der Sitzung** (*successive relearning*) | Falsch beantwortete Übungen kommen 3 Karten später noch einmal. | Abrufen bis zum Erfolg, über mehrere Sitzungen verteilt, ist eine besonders wirksame Kombination (Rawson & Dunlosky). |
| **Explizites, sofortiges Feedback mit Regel** | Nach jeder Antwort gibt es die Lösung, den ganzen Satz und die Regel. Bei Fehlern erklärt die **Fehlerdiagnose**, warum genau *diese* Antwort falsch ist: falsche Person, vergessener Stammwechsel, falsche Verbklasse, ser statt estar, Genus/Zahl, fehlende oder überflüssige Wörter. Akzente, die die Bedeutung ändern (esta/está, tu/tú), zählen als Fehler; andere fehlende Akzente als „fast richtig“. | Explizites Feedback mit metalinguistischer Erklärung wirkt besser als implizites (z. B. Ellis, Loewen & Erlam 2006). Meta-Analysen bescheinigen Korrekturfeedback insgesamt einen positiven Effekt. |
| **Korrektur selbst produzieren** (*uptake/repair*) | Nach einem Fehler tippst du die Lösung einmal ab. | Selbst produzierte Korrekturen bleiben besser hängen als nur gelesene. |
| **Verschachteln** (*interleaving*) | Der tägliche Mix mischt Wiederholungen aus allen Themen mit Neuem. Verwechselbare Paare wie ser/estar, hay/estar und gusta/gustan werden direkt gegenübergestellt. | Hilft vor allem bei ähnlich aussehenden Formen. Die Befunde zur Grammatik sind gemischt, aber eher positiv. Fühlt sich schwerer an, wirkt aber länger. |
| **Bedeutung im Kontext** | Ganze, alltagsnahe Sätze mit deutscher Übersetzung statt isolierter Formen. Spanische Lösungen werden vorgelesen (Web Speech API). | Form-Bedeutungs-Verknüpfung ist zentral für Grammatikerwerb. Hören stützt die Aussprache. |
| **Motivation: Kompetenz, Autonomie, kleine Spielelemente** | XP, Tages-Serie 🔥, Tagesziel-Ring, Kombo-Zähler, Sterne pro Thema, 60-s-Verb-Sprint mit Rekord. Alle Themen sind frei wählbar, ein Vorschlag ist markiert. | Gamification-Meta-Analysen finden kleine bis mittlere positive Effekte auf Motivation und Leistung, am stärksten, wenn Autonomie und Kompetenzerleben gestärkt werden. Punkte allein reichen nicht. |
| **Automatisierung** | Der Verb-Sprint trainiert schnellen Abruf der Konjugation unter leichtem Zeitdruck. | Wiederholtes, schnelles Abrufen fördert die Prozeduralisierung (Skill-Acquisition-Theorie, DeKeyser). |

**Empfehlung:** Täglich 5–10 Minuten im „Täglichen Mix“, dazu ab und zu ein Sprint. Kurze, regelmäßige Einheiten schlagen lange Sitzungen.

## Inhalte erweitern

Die Übungen liegen in [`content/a1.json`](content/a1.json) und werden beim Start von dort geladen. Fehlt die Datei, nimmt der Server die eingebettete Version. Typen von Übungen:

- `cloze`: Lücke `___` im `prompt`, `hint` erscheint in Klammern
- `choice`: `options` (die richtige muss in `answers` stehen)
- `order`: Wortkacheln, die aus `answers[0]` erzeugt werden
- `translate`: deutscher `prompt`, mehrere akzeptierte `answers`

Groß-/Kleinschreibung, Satzzeichen und überzählige Leerzeichen werden beim Prüfen ignoriert. Bei Auswahlaufgaben braucht jede falsche Option eine Begründung in `why` (`"why": {"Option": "Darum falsch …"}`), sonst startet der Server nicht. Freie Eingaben erklärt die automatische Diagnose ([`src/diagnose.rs`](src/diagnose.rs)); `why` kann dort zusätzlich typische Fehler abdecken.

Alle Übungen mit automatisch erzeugten Falschantworten prüfen: `cargo test --release audit -- --ignored --nocapture` Die Dateien in `static/` werden ebenfalls von der Platte gelesen, sodass Änderungen ohne Neukompilieren wirken.

## Aufbau

```
src/main.rs     HTTP-Routen, Zusammenstellung der Sitzung (fällig → neu → schwach, verschachtelt)
src/srs.rs      Fortschritt, Leitner-Boxen, XP, Serie
src/check.rs    Antwortprüfung (tolerant, Akzent-Erkennung)
src/diagnose.rs Fehlerdiagnose: warum eine konkrete Antwort falsch ist
src/tts.rs      Sprachausgabe (Google Chirp 3 HD, Dateien in audio/)
src/verbs.rs    Konjugationstabellen für den Sprint
content/a1.json Themen, Regelkarten, Übungen
static/         Oberfläche (Vanilla JS, hell/dunkel)
```

Tests: `cargo test`

## Quellen (Auswahl)

- Kim & Webb (2022), Meta-Analyse zu verteiltem Üben beim L2-Lernen: <https://ir.lib.uwo.ca/etd/8600/>
- ANR-Projekt GLITCH zu Abruf- und Verteilungseffekten speziell beim Grammatiklernen: <https://anr.fr/Project-ANR-23-CE28-0026>
- Ellis, Loewen & Erlam (2006), implizites vs. explizites Korrekturfeedback: <https://researchspace.auckland.ac.nz/handle/2292/13170>
- Interleaving und L2-Grammatik (Arbeitsgedächtnis, Prozeduralisierung): <https://sage.figshare.com/collections/The_role_of_working_memory_in_blocked_and_interleaved_grammar_practice_Proceduralization_of_L2_syntax/4925595>
- Gamification und Motivation, Meta-Analyse: <https://encyclopedia.pub/entry/history/show/107433>
