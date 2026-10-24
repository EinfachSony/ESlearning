//! Spanische Sprachausgabe über Google Cloud Text-to-Speech.
//!
//! Audiodateien liegen im Repo unter `audio/<Stimme>/` und werden mit
//! `cargo run --release -- --audio` vorab für alle Übungssätze erzeugt. Der Server
//! fragt Google nur noch an, wenn ein Satz fehlt (z. B. neu hinzugefügte Übungen).
//! Der Schlüssel kommt aus der Umgebung (`GOOGLE_API_KEY`), einer `.env`-Datei oder `TTS_Key.txt`.

use base64::Engine;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;

const ENDPOINT: &str = "https://texttospeech.googleapis.com/v1/text:synthesize";
const AUDIO_DIR: &str = "audio";

/// (Kennung im Frontend, Google-Stimme)
pub const VOICES: [(&str, &str); 2] = [("female", "es-ES-Chirp3-HD-Kore"), ("male", "es-ES-Chirp3-HD-Charon")];

pub struct Tts {
    key: Option<String>,
    client: reqwest::Client,
}

/// Minimaler `.env`-Leser: `NAME=wert`, Kommentare mit `#`, optionale Anführungszeichen.
fn dotenv(name: &str) -> Option<String> {
    let src = std::fs::read_to_string(".env").ok()?;
    src.lines().find_map(|line| {
        let (k, v) = line.trim().split_once('=')?;
        (k.trim() == name).then(|| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
    })
}

/// `TTS_Key.txt`: erste nicht-leere Zeile, wahlweise als `NAME=wert`.
fn key_file() -> Option<String> {
    let src = std::fs::read_to_string("TTS_Key.txt").ok()?;
    let line = src.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#'))?;
    let value = line.split_once('=').map_or(line, |(_, v)| v.trim());
    Some(value.trim_matches(|c| c == '"' || c == '\'').to_string()).filter(|v| !v.is_empty())
}

fn setting(name: &str) -> Option<String> {
    std::env::var(name).ok().or_else(|| dotenv(name)).filter(|v| !v.is_empty())
}

/// Google-Stimmenname zur Frontend-Kennung; unbekannt → Frauenstimme.
pub fn voice_name(id: &str) -> &'static str {
    VOICES.iter().find(|(k, _)| *k == id).map_or(VOICES[0].1, |(_, v)| v)
}

fn audio_path(voice: &str, text: &str) -> PathBuf {
    let hash = Sha256::digest(text.as_bytes());
    let name: String = hash.iter().take(12).map(|b| format!("{b:02x}")).collect();
    PathBuf::from(AUDIO_DIR).join(voice).join(format!("{name}.mp3"))
}

impl Tts {
    pub fn from_env() -> Self {
        let key = setting("GOOGLE_API_KEY").or_else(|| setting("GOOGLE_TTS_API_KEY")).or_else(key_file);
        let files = std::path::Path::new(AUDIO_DIR).is_dir();
        match (&key, files) {
            (Some(_), _) => println!("Sprachausgabe: Google TTS (Dateien in {AUDIO_DIR}/, fehlende werden nachgeladen)"),
            (None, true) => println!("Sprachausgabe: Google TTS (nur vorhandene Dateien in {AUDIO_DIR}/)"),
            (None, false) => println!("Sprachausgabe: Browser-Stimme (kein Schlüssel, kein {AUDIO_DIR}/-Ordner)"),
        }
        Tts { key, client: reqwest::Client::new() }
    }

    pub fn available(&self) -> bool {
        self.key.is_some() || std::path::Path::new(AUDIO_DIR).is_dir()
    }

    /// MP3 für den Text: aus `audio/`, sonst einmalig von Google holen und dort ablegen.
    pub async fn speak(&self, voice: &str, text: &str) -> Result<Vec<u8>, String> {
        let path = audio_path(voice, text);
        if let Ok(bytes) = tokio::fs::read(&path).await {
            return Ok(bytes);
        }
        let bytes = self.synthesize(voice, text).await?;
        if let Some(dir) = path.parent() {
            if tokio::fs::create_dir_all(dir).await.is_ok() {
                let _ = tokio::fs::write(&path, &bytes).await;
            }
        }
        Ok(bytes)
    }

    async fn synthesize(&self, voice: &str, text: &str) -> Result<Vec<u8>, String> {
        let key = self.key.as_deref().ok_or("Audio fehlt und kein API-Schlüssel vorhanden")?;
        let body = json!({
            "input": { "text": text },
            "voice": { "languageCode": "es-ES", "name": voice },
            "audioConfig": { "audioEncoding": "MP3", "speakingRate": 0.92 }
        });
        let resp = self
            .client
            .post(ENDPOINT)
            .header("x-goog-api-key", key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Google TTS nicht erreichbar: {e}"))?;
        let status = resp.status();
        let data: serde_json::Value = resp.json().await.map_err(|e| format!("Google TTS: ungültige Antwort: {e}"))?;
        if !status.is_success() {
            let msg = data["error"]["message"].as_str().unwrap_or("unbekannter Fehler");
            return Err(format!("Google TTS {status}: {msg}"));
        }
        let audio = data["audioContent"].as_str().ok_or("Google TTS: kein audioContent")?;
        base64::engine::general_purpose::STANDARD
            .decode(audio)
            .map_err(|e| format!("Google TTS: Audio nicht lesbar: {e}"))
    }
}

/// Erzeugt alle fehlenden Audiodateien für beide Stimmen (8 Anfragen parallel).
pub async fn generate_all(tts: Tts, texts: Vec<String>) {
    if tts.key.is_none() {
        eprintln!("Kein API-Schlüssel gefunden (GOOGLE_API_KEY, .env oder TTS_Key.txt).");
        std::process::exit(1);
    }
    let tts = Arc::new(tts);
    let missing = || -> Vec<(&'static str, String)> {
        VOICES
            .iter()
            .flat_map(|(_, v)| texts.iter().map(move |t| (*v, t.clone())))
            .filter(|(v, t)| !audio_path(v, t).exists())
            .collect()
    };
    let total = missing().len();
    println!("{} Sätze × {} Stimmen, davon {total} noch zu erzeugen …", texts.len(), VOICES.len());

    // Google begrenzt die Anfragen pro Minute: bei 429 eine Minute warten und weitermachen.
    let (mut ok, mut failed) = (0, 0);
    for round in 0..15 {
        let jobs = missing();
        if jobs.is_empty() {
            break;
        }
        if round > 0 {
            println!("\nAnfragelimit erreicht, warte 60 s …");
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
        let mut limited = false;
        for chunk in jobs.chunks(8) {
            let mut set = tokio::task::JoinSet::new();
            for (voice, text) in chunk.iter().cloned() {
                let tts = tts.clone();
                set.spawn(async move { tts.speak(voice, &text).await.map_err(|e| format!("{text}: {e}")) });
            }
            while let Some(res) = set.join_next().await {
                match res {
                    Ok(Ok(_)) => ok += 1,
                    Ok(Err(e)) if e.contains("429") => limited = true,
                    Ok(Err(e)) => {
                        failed += 1;
                        eprintln!("\n{e}");
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("\n{e}");
                    }
                }
            }
            print!("\r{ok}/{total} erzeugt");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            if limited {
                break;
            }
        }
        if !limited {
            break;
        }
    }
    println!("\nFertig: {ok} erzeugt, {failed} fehlgeschlagen, {} fehlen noch.", missing().len());
    println!("Audio ist bereit. Server starten mit:  cargo run --release");
}
