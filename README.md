# ESlearning — Sonys Spanischabenteuer

Website: https://einfachsony.github.io/ESlearning/

Argentinisches Spanisch mit vos und ustedes, 224 kurzen Theorieabschnitten und 441 Übungen. Vier Aufgabenarten: frei übersetzen, mehrere Lücken im vollständigen Satz ergänzen, Hörverständnis und Mini-Missionen mit zwei Sätzen. Alle Antworten werden getippt.

Theorie bringt einmalig XP und Gold; vollständige Kapitel geben einen Bonus. Spielerlevel, Titel, tägliche Quests, Erfolge, Combos und freischaltbare Designs ergänzen verteilte Wiederholung. Fortschritt wird im Browser gespeichert und kann exportiert werden.

## Audio

Die männliche argentinische Neuralstimme es-AR-TomasNeural wird beim Build für sämtliche Übungssätze erzeugt (edge-tts 7.3.1, imageio-ffmpeg 0.6.0). Der Build schlägt bei unvollständigen Aufnahmen fehl, statt eine schlechtere oder anderssprachige Ersatzstimme zu veröffentlichen. WAV-Dateien bewahren das decodierte Audio ohne erneute verlustbehaftete Kodierung und haben 0,4 Sekunden Ausklang. Die Dateien werden im Actions-Cache gehalten. Die Generierung benötigt den externen Microsoft-Edge-Sprachdienst. Eine absolute Qualitätsrangfolge ist nicht belegt.

## Verzeichnisstruktur

web/ enthält die GitHub-Pages-Fassung, scripts/ den Audiogenerator, source/ den ursprünglichen Rust-Quellcode der hochgeladenen Version 0. Pages benötigt keinen Rust-Server und keinen privaten API-Schlüssel.

## Minigames und Expedition

Der Einstieg erfolgt über die Südamerika-Expedition oder #adventure. Sechs Orte bilden eine fortlaufende Argentinien-Kampagne mit je einem Gespräch und einem dreiphasigen Bosskampf (zwölf Missionen). Im Trainingslager gibt es Funkmissionen, Satzschmiede, Lagerverteidigung und Magierprüfung. Die Prüfung setzt ein vollständig gelesenes Theoriekapitel voraus. Alle Antworten werden getippt.

Gespräche haben mehrere explizit akzeptierte Formulierungen, einschließlich optionaler Subjektpronomen. Buenos Aires hat einen Getränke-Zweig mit unterschiedlicher Folgefrage. Die Auswertung verwendet redaktionelle Musterantworten, keine generative KI; sie erkennt daher nicht jede mögliche korrekte Paraphrase.

Erstabschlüsse der Reise geben XP, Gold, einen Talentpunkt und ein festes Erinnerungsstück. Sechs Talente gewähren zusätzliche Schutzpunkte, langsameres Audio oder einen ersten Tipp ohne Sternenabzug. Wiederholungen können die Bestwertung verbessern; Reise-Erstbelohnungen werden nicht doppelt vergeben. Trainingsbelohnungen sind je Spielart einmal pro Tag verfügbar. Eingebundene Grammatikaufgaben aktualisieren zugleich die bestehende Wiederholungsplanung. Aktive Runde, Sammlung, Ausrüstung und Talente werden mit dem bestehenden Fortschritt lokal gespeichert und exportiert.

Prüfungen: node scripts/verify_learning.cjs web und node scripts/verify_adventure.cjs web. Der Pages-Workflow führt beide aus und erzeugt die neuen Dialogaufnahmen mit derselben männlichen argentinischen Neuralstimme.

## WoW original sound profile (default)

The default profile uses four original World of Warcraft recordings selected from the user-provided Wowhead sound database. Audio copyright remains with Blizzard Entertainment; ESlearning is independent and makes no claim of ownership or affiliation. Exact source pages, public download URLs and immutable SHA-256 hashes are in `web/wow-sounds.json`.

- Correct answers, theory parts and combo feedback: LootCoinSmall, 0.635 seconds, reduced gain.
- Completed series and daily goals: QUESTCOMPLETED, 8.348 seconds.
- Player level up: LEVELUP, 4.365 seconds.
- Boss clears and major achievements: UI_Alert_AchievementGained, 5.187 seconds.

Original pitch and complete tails are preserved. Combos in this profile use restrained gain emphasis at milestones rather than synthetic melodies or pitch-shifted game recordings. Wrong answers and anticipation cues remain visual without the former synthesized tones. The original synthetic background music is off by default, including a one-time migration of older audio settings. It can be enabled explicitly. All volume, speech-ducking, reward priority and haptic controls remain available.

`scripts/prepare_wow_audio.py` retrieves the four public Wowhead downloads at build time, verifies hashes and creates lossless decoded PCM WAV alternatives with the existing FFmpeg dependency. Build failure preserves the preceding deployment. `web/wow-sound-bank.js` preloads same-origin WAV files and decodes each once into the shared AudioContext; the browser has no runtime dependency on Wowhead CORS or third-party availability. An unavailable clip is reported in the audio panel and does not silently revert to the disliked synthesized profile. No original binary recordings are committed to Git; the build prepares them in the Pages artifact.

Open “♫ Klang” to select World of Warcraft / synthetic Fantasy and preview coin, quest, level and achievement sounds. Sound credit and individual source links are available in that dialog. `node scripts/verify_wow_audio.cjs web` checks source hashes, non-silent PCM, full clip durations, preload/decode caching, cue mapping and load failure.

## Alternative synthetic Fantasy audio profile

Original procedural fantasy score: warm crystal/bell transients, plucked harp, soft additive horns, flute/pads and tuned percussion. All notes and waveforms are authored here; no third-party samples, copied melodies or additional runtime packages. D major / B minor, 80 BPM, 24 kHz stereo PCM. Existing male es-AR-TomasNeural WAV recordings and their padded tails remain unchanged.

- `web/sound-score.js`: deterministic composition and additive PCM renderer, shared by browser, worker and signal tests.
- `web/sound-worker.js`: prepares frequently used effects and five 24-second stems outside the UI thread.
- `web/reward-audio.js`: single Web Audio graph, cached buffers, gain ramps, compression, bounded effect crossfades, priority/coalescing, speech ducking and lifecycle.
- `web/reward-audio.css`: accessible native-dialog controls and reduced-motion treatment.
- `web/local-api.js`: factual result events and persistent, unique learning milestone claims; adventure answers emit once. No reward is replayed by rendering/resuming.

Open “♫ Klang” in the header. Master, music, effects, voice, dynamic music, combos, haptics, sensory mode and intensity persist under `eslearning.audio.v1`. Klangserie persists within the tab session under `eslearning.combo`; an error reduces it by two rather than erasing it. Existing XP, theory and SRS remain in `gramatica.progress.v1`. Optional audio preferences are device-specific; the existing progress export preserves factual audio-learning milestone history.

Effects start from prepared AudioBuffers. One foreground effect and at most one short retiring crossfade are allowed; an important reward replaces less important pending rewards. Fanfare requests within 240 ms coalesce. Speech ducks music to 12% and effects to 6%; major rewards wait until speech ends. No fanfare backlog. Music starts all five looping stems at one AudioContext time, adds layers at 20/40/60/80% and resolves through the completion cue. Hidden tabs pause voice, drop pending cues, stop music and suspend the context; a fresh gesture resumes audio. Autoplay always requires browser permission/user interaction. Mute and gain changes use smoothing.

Small, medium, large and legendary cues span approximately 0.17–0.5 / 0.7–1.6 / 2.4–3.4 / 5.4–6.4 seconds. Combo milestones develop a shared motif at 5, 10, 15 and 20. Rare cues use actual distinct two-sentence exercises (50), 100 exercises at Leitner box 4+, all exercises of a grammar topic at box 4+, or a correct unaided recall at least seven days after its last correct recall. These are exercise-based measures, not claims of 100 unique vocabulary words or certified CEFR proficiency. Long-term measurement begins with this release and does not invent older timestamps.

Haptics are opt-in through navigator.vibrate and unavailable on some browsers, including many iOS browsers. Reduced-motion preference defaults to sensory mode; sensory mode suppresses music, haptics and confetti. Test previews let users calibrate volume with their own headphones; no code can guarantee the loudness of external hardware.

Additional checks: `node scripts/verify_reward_audio.cjs web` (PCM energy, peaks, tails, 100 answers, interruption, bounded overlap, music, voice, settings) and `node scripts/verify_audio_learning.cjs web` (unique factual milestones, seven-day recall, resume and adventure event deduplication). Browser/manual device and headphone listening remains necessary for subjective timbre and hardware-specific behavior.

## Local Alliance-capital background music

The city-music extension does not download, bundle or publish complete copyrighted zone-music tracks. Users select their own music files for Sturmwind, Eisenschmiede and Darnassus in “♫ Klang → Musikdateien verwalten”. `web/city-music.js` stores these files locally in IndexedDB (`eslearning.city-music`, `tracks` store), never uploads them, and uses local blob URLs through MediaElementAudioSourceNodes into the existing music bus. The same master/music controls and 12% speech ducking therefore apply to city music.

Multiple files per city are supported. The Alliance playlist interleaves available cities deterministically. Up to two active music streams crossfade over approximately one second; stale asynchronous playback requests cannot take over. Playback continues between home, theory and exercises, pauses when the tab is hidden, and resumes only after another interaction. Music scene and enable state persist in the audio settings, while current track/position persist in `eslearning.city-music.position.v1`. Audio files remain outside the learning-progress export; users must retain their original files. Import supports common browser-playable audio formats, limits individual files to 30 MB and total local music to 200 MB, and reports storage/format failures visibly. Removing a local copy never deletes its original file.

No extra dependency is required. `node scripts/verify_city_music.cjs web` checks import/storage, duplicate avoidance, file limits, playlist ordering, fades, pause/resume, stale requests, resource cleanup and failed playback. The prior synthetic music remains available as an explicit alternative. No capital music is falsely presented as ready before the user imports it.
