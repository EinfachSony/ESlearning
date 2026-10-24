# ESlearning — Sonys Spanischabenteuer

Website: https://einfachsony.github.io/ESlearning/

Argentinisches Spanisch mit vos und ustedes, 224 kurzen Theorieabschnitten und 441 Übungen. Vier Aufgabenarten: frei übersetzen, mehrere Lücken im vollständigen Satz ergänzen, Hörverständnis und Mini-Missionen mit zwei Sätzen. Alle Antworten werden getippt.

Theorie bringt einmalig XP und Gold; vollständige Kapitel geben einen Bonus. Spielerlevel, Titel, tägliche Quests, Erfolge, Combos und freischaltbare Designs ergänzen verteilte Wiederholung. Fortschritt wird im Browser gespeichert und kann exportiert werden.

## Audio

Die männliche argentinische Neuralstimme es-AR-TomasNeural wird beim Build für sämtliche Übungssätze erzeugt (edge-tts 7.3.1, imageio-ffmpeg 0.6.0). Der Build schlägt bei unvollständigen Aufnahmen fehl, statt eine schlechtere oder anderssprachige Ersatzstimme zu veröffentlichen. WAV-Dateien bewahren das decodierte Audio ohne erneute verlustbehaftete Kodierung und haben 0,4 Sekunden Ausklang. Die Dateien werden im Actions-Cache gehalten. Die Generierung benötigt den externen Microsoft-Edge-Sprachdienst. Eine absolute Qualitätsrangfolge ist nicht belegt.

## Verzeichnisstruktur

web/ enthält die GitHub-Pages-Fassung, scripts/ den Audiogenerator, source/ den ursprünglichen Rust-Quellcode der hochgeladenen Version 0. Pages benötigt keinen Rust-Server und keinen privaten API-Schlüssel.
