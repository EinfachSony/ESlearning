//! Gramática – lokaler Spanisch-Grammatiktrainer.
//!
//! Start: `cargo run --release`, dann http://127.0.0.1:3000 öffnen.

mod check;
mod diagnose;
mod content;
mod srs;
mod tts;
mod verbs;

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use check::Verdict;
use content::{Content, Item, Kind};
use serde::{Deserialize, Serialize};
use srs::{ItemState, Progress, MASTERED_BOX};
use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

struct AppState {
    content: Content,
    progress: Mutex<Progress>,
    rng: Mutex<u64>,
    tts: tts::Tts,
}

type Shared = Arc<AppState>;

impl AppState {
    fn rand(&self) -> u64 {
        let mut s = self.rng.lock().unwrap();
        // xorshift64
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        *s
    }

    fn shuffle<T>(&self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            v.swap(i, (self.rand() % (i as u64 + 1)) as usize);
        }
    }
}

// ---------- Übersicht ----------

#[derive(Serialize)]
struct Stats {
    xp: u64,
    today_xp: u64,
    daily_goal: u64,
    streak: u32,
    sprint_best: u32,
    due_total: usize,
    level: String,
    /// Google-Sprachausgabe verfügbar (sonst Browser-Stimme).
    google_tts: bool,
}

#[derive(Serialize)]
struct TopicInfo {
    id: String,
    title: String,
    title_es: String,
    summary: String,
    explanation: String,
    details: String,
    total: usize,
    seen: usize,
    due: usize,
    /// 0–100: wie gut die Übungen des Themas sitzen.
    mastery: u32,
}

#[derive(Serialize)]
struct Overview {
    stats: Stats,
    topics: Vec<TopicInfo>,
    recommended: Option<String>,
}

fn mastery(items: &[Item], p: &Progress) -> u32 {
    if items.is_empty() {
        return 0;
    }
    let sum: u32 = items
        .iter()
        .map(|i| p.items.get(&i.id).map_or(0, |s| s.level.min(MASTERED_BOX)) as u32)
        .sum();
    sum * 100 / (items.len() as u32 * MASTERED_BOX as u32)
}

fn is_due(s: Option<&ItemState>, now: i64) -> bool {
    s.is_some_and(|s| s.due <= now)
}

/// Erstes Thema (in Lernreihenfolge) mit noch unbekannten Übungen.
fn recommended_topic<'a>(c: &'a Content, p: &Progress) -> Option<&'a content::Topic> {
    c.topics.iter().find(|t| t.items.iter().any(|i| !p.items.contains_key(&i.id)))
}

async fn overview(State(app): State<Shared>) -> Json<Overview> {
    let p = app.progress.lock().unwrap();
    let now = srs::now();
    let topics: Vec<TopicInfo> = app
        .content
        .topics
        .iter()
        .map(|t| TopicInfo {
            id: t.id.clone(),
            title: t.title.clone(),
            title_es: t.title_es.clone(),
            summary: t.summary.clone(),
            explanation: t.explanation.clone(),
            details: t.details.clone(),
            total: t.items.len(),
            seen: t.items.iter().filter(|i| p.items.contains_key(&i.id)).count(),
            due: t.items.iter().filter(|i| is_due(p.items.get(&i.id), now)).count(),
            mastery: mastery(&t.items, &p),
        })
        .collect();
    Json(Overview {
        stats: Stats {
            xp: p.xp,
            today_xp: p.today_xp(),
            daily_goal: p.daily_goal,
            streak: p.streak(),
            sprint_best: p.sprint_best,
            due_total: topics.iter().map(|t| t.due).sum(),
            level: app.content.level.clone(),
            google_tts: app.tts.available(),
        },
        recommended: recommended_topic(&app.content, &p).map(|t| t.id.clone()),
        topics,
    })
}

// ---------- Sitzung ----------

#[derive(Deserialize)]
struct SessionQuery {
    topic: Option<String>,
    size: Option<usize>,
}

#[derive(Serialize)]
struct PublicItem {
    id: String,
    topic: String,
    topic_title: String,
    kind: Kind,
    prompt: String,
    hint: Option<String>,
    options: Vec<String>,
    words: Vec<String>,
    de: Option<String>,
    is_new: bool,
    level: u8,
}

#[derive(Serialize)]
struct Session {
    /// Thema, dessen Regelkarte vor der Übung gezeigt werden soll.
    intro: Option<String>,
    items: Vec<PublicItem>,
}

/// Wortkacheln für Satzbau-Übungen. Das erste Wort wird kleingeschrieben,
/// damit die Großschreibung den Satzanfang nicht verrät.
fn order_words(app: &AppState, answer: &str) -> Vec<String> {
    let mut words: Vec<String> = answer.split_whitespace().map(String::from).collect();
    if let Some(first) = words.first_mut() {
        *first = first.to_lowercase();
    }
    let original = words.clone();
    for _ in 0..5 {
        app.shuffle(&mut words);
        if words != original || words.len() < 2 {
            break;
        }
    }
    words
}

fn public(app: &AppState, topic: &content::Topic, item: &Item, st: Option<&ItemState>) -> PublicItem {
    let mut options = item.options.clone();
    app.shuffle(&mut options);
    PublicItem {
        id: item.id.clone(),
        topic: topic.id.clone(),
        topic_title: topic.title.clone(),
        kind: item.kind,
        prompt: item.prompt.clone(),
        hint: item.hint.clone(),
        options,
        words: if item.kind == Kind::Order { order_words(app, &item.answers[0]) } else { vec![] },
        de: item.de.clone(),
        is_new: st.is_none(),
        level: st.map_or(0, |s| s.level),
    }
}

async fn session(State(app): State<Shared>, Query(q): Query<SessionQuery>) -> Result<Json<Session>, StatusCode> {
    let size = q.size.unwrap_or(15).clamp(5, 50);
    let p = app.progress.lock().unwrap();
    let now = srs::now();

    let focus = match &q.topic {
        Some(id) => Some(app.content.topics.iter().find(|t| &t.id == id).ok_or(StatusCode::NOT_FOUND)?),
        None => recommended_topic(&app.content, &p),
    };
    let pool: Vec<(&content::Topic, &Item)> = app
        .content
        .topics
        .iter()
        .filter(|t| q.topic.is_none() || Some(t.id.as_str()) == q.topic.as_deref())
        .flat_map(|t| t.items.iter().map(move |i| (t, i)))
        .collect();

    // 1. Fällige Wiederholungen – die überfälligsten zuerst.
    let mut due: Vec<_> = pool.iter().filter(|(_, i)| is_due(p.items.get(&i.id), now)).copied().collect();
    due.sort_by_key(|(_, i)| p.items[&i.id].due);
    // 2. Neue Übungen aus dem Fokus-Thema, in der vorgesehenen Reihenfolge (leicht → schwer).
    let new: Vec<_> = focus
        .map(|t| t.items.iter().filter(|i| !p.items.contains_key(&i.id)).map(|i| (t, i)).collect())
        .unwrap_or_default();
    // Mindestens ein paar neue Übungen, damit es auch bei vielen Wiederholungen weitergeht.
    let min_new = new.len().min(if q.topic.is_some() { size / 2 } else { 4 });
    due.truncate(size.saturating_sub(min_new));
    // Im Mix höchstens 10 neue – der Rest kommt aus älteren Themen (Interleaving).
    let new_cap = if q.topic.is_some() { size } else { 10 };
    let new: Vec<_> = new.into_iter().take((size - due.len()).min(new_cap)).collect();

    // 3. Auffüllen mit den schwächsten bereits gesehenen Übungen.
    let mut extra = Vec::new();
    if due.len() + new.len() < size {
        let mut weak: Vec<_> = pool
            .iter()
            .filter(|(_, i)| p.items.get(&i.id).is_some_and(|s| s.due > now))
            .copied()
            .collect();
        weak.sort_by_key(|(_, i)| {
            let s = &p.items[&i.id];
            (s.level, s.due)
        });
        extra = weak.into_iter().take(size - due.len() - new.len()).collect();
    }

    // Verschachteln (Interleaving): Wiederholungen gemischt, neue Übungen bleiben in
    // Reihenfolge und werden zufällig dazwischen verteilt. Die ersten zwei neuen kommen
    // direkt nach der Regelkarte.
    let mut reviews: Vec<_> = due.into_iter().chain(extra).collect();
    app.shuffle(&mut reviews);
    let lead = new.len().min(2);
    let mut out: Vec<_> = new[..lead].to_vec();
    let (mut ni, mut ri) = (lead, 0);
    while ni < new.len() || ri < reviews.len() {
        let (nl, rl) = ((new.len() - ni) as u64, (reviews.len() - ri) as u64);
        if app.rand() % (nl + rl) < nl {
            out.push(new[ni]);
            ni += 1;
        } else {
            out.push(reviews[ri]);
            ri += 1;
        }
    }

    let intro = focus
        .filter(|t| !t.items.iter().any(|i| p.items.contains_key(&i.id)) && lead > 0)
        .map(|t| t.id.clone());
    let items = out.into_iter().map(|(t, i)| public(&app, t, i, p.items.get(&i.id))).collect();
    Ok(Json(Session { intro, items }))
}

// ---------- Antworten ----------

#[derive(Deserialize)]
struct CheckReq {
    id: String,
    answer: String,
    #[serde(default)]
    hint_used: bool,
    #[serde(default)]
    relearn: bool,
    #[serde(default)]
    combo: u32,
}

#[derive(Serialize)]
struct CheckResp {
    verdict: Verdict,
    expected: String,
    /// Vollständiger spanischer Satz (für Vorlesen & Anzeige).
    sentence: String,
    explain: Option<String>,
    /// Warum genau diese Antwort falsch ist.
    notes: Vec<String>,
    de: Option<String>,
    xp_gained: u64,
    level: u8,
    stats: StatsLite,
}

#[derive(Serialize)]
struct StatsLite {
    xp: u64,
    today_xp: u64,
    daily_goal: u64,
    streak: u32,
}

fn stats_lite(p: &Progress) -> StatsLite {
    StatsLite { xp: p.xp, today_xp: p.today_xp(), daily_goal: p.daily_goal, streak: p.streak() }
}

fn full_sentence(item: &Item) -> String {
    let answer = &item.answers[0];
    match item.kind {
        Kind::Cloze | Kind::Choice if item.prompt.contains("___") => {
            // Deutsche Zusätze in Klammern entfernen, Pfeile als Pause lesen.
            let mut s = item.prompt.replacen("___", answer, 1).replace(" → ", ", ");
            while let (Some(a), Some(b)) = (s.find(" ("), s.find(')')) {
                if b < a {
                    break;
                }
                s.replace_range(a..=b, "");
            }
            s
        }
        _ => answer.clone(),
    }
}

async fn check_answer(State(app): State<Shared>, Json(req): Json<CheckReq>) -> Result<Json<CheckResp>, StatusCode> {
    let (_, item) = app.content.find(&req.id).ok_or(StatusCode::NOT_FOUND)?;
    let verdict = check::check(&req.answer, &item.answers);
    let (verdict, notes) = diagnose::analyze(item, &req.answer, verdict);
    let mut p = app.progress.lock().unwrap();
    let out = p.record(&item.id, verdict, req.hint_used, req.relearn, req.combo);
    Ok(Json(CheckResp {
        verdict,
        expected: item.answers[0].clone(),
        sentence: full_sentence(item),
        explain: item.explain.clone(),
        notes,
        de: item.de.clone().or_else(|| matches!(item.kind, Kind::Translate | Kind::Order).then(|| item.prompt.clone())),
        xp_gained: out.xp_gained,
        level: out.level,
        stats: stats_lite(&p),
    }))
}

#[derive(Deserialize)]
struct HintReq {
    id: String,
}

#[derive(Serialize)]
struct HintResp {
    /// Bei Auswahl: eine falsche Option, die ausgeblendet wird.
    remove: Option<String>,
    /// Sonst: Anfang der Lösung.
    text: Option<String>,
}

async fn hint(State(app): State<Shared>, Json(req): Json<HintReq>) -> Result<Json<HintResp>, StatusCode> {
    let (_, item) = app.content.find(&req.id).ok_or(StatusCode::NOT_FOUND)?;
    let answer = &item.answers[0];
    Ok(Json(match item.kind {
        Kind::Choice => {
            let wrong: Vec<_> = item.options.iter().filter(|o| *o != answer).collect();
            let pick = wrong[(app.rand() % wrong.len() as u64) as usize].clone();
            HintResp { remove: Some(pick), text: None }
        }
        Kind::Cloze => {
            let n = (answer.chars().count() / 3).max(1);
            HintResp { remove: None, text: Some(format!("{}…", answer.chars().take(n).collect::<String>())) }
        }
        Kind::Order | Kind::Translate => {
            let first = answer.split_whitespace().next().unwrap_or("");
            HintResp { remove: None, text: Some(format!("{first} …")) }
        }
    }))
}

// ---------- Verb-Sprint & Einstellungen ----------

#[derive(Deserialize)]
struct ExplainReq {
    given: String,
    expected: String,
}

async fn explain(Json(req): Json<ExplainReq>) -> Json<Vec<String>> {
    Json(diagnose::explain_word(&req.given, &req.expected))
}

async fn verb_list() -> Json<Vec<verbs::Verb>> {
    Json(verbs::all())
}

#[derive(Deserialize)]
struct SprintReq {
    score: u32,
}

#[derive(Serialize)]
struct SprintResp {
    best: u32,
    new_record: bool,
    xp_gained: u64,
    stats: StatsLite,
}

async fn sprint_done(State(app): State<Shared>, Json(req): Json<SprintReq>) -> Json<SprintResp> {
    let mut p = app.progress.lock().unwrap();
    let new_record = req.score > p.sprint_best;
    if new_record {
        p.sprint_best = req.score;
    }
    let xp = u64::from(req.score.min(200)) * 2;
    p.add_xp(xp);
    p.save();
    Json(SprintResp { best: p.sprint_best, new_record, xp_gained: xp, stats: stats_lite(&p) })
}

#[derive(Deserialize)]
struct GoalReq {
    goal: u64,
}

async fn set_goal(State(app): State<Shared>, Json(req): Json<GoalReq>) -> Json<StatsLite> {
    let mut p = app.progress.lock().unwrap();
    p.daily_goal = req.goal.clamp(20, 2000);
    p.save();
    Json(stats_lite(&p))
}

// ---------- Sprachausgabe ----------

#[derive(Deserialize)]
struct TtsQuery {
    text: String,
    /// "female" oder "male"
    #[serde(default)]
    voice: String,
}

async fn tts(State(app): State<Shared>, Query(q): Query<TtsQuery>) -> Response {
    let text = q.text.trim();
    if text.is_empty() || text.chars().count() > 300 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match app.tts.speak(tts::voice_name(&q.voice), text).await {
        Ok(mp3) => ([(header::CONTENT_TYPE, "audio/mpeg"), (header::CACHE_CONTROL, "max-age=31536000")], mp3).into_response(),
        Err(e) => {
            eprintln!("{e}");
            (StatusCode::BAD_GATEWAY, e).into_response()
        }
    }
}

// ---------- Statische Dateien ----------

/// Liefert Dateien aus `static/` (falls vorhanden, praktisch beim Anpassen), sonst die eingebettete Version.
fn asset(name: &str, embedded: &'static str, mime: &'static str) -> Response {
    let body = std::fs::read_to_string(format!("static/{name}")).unwrap_or_else(|_| embedded.to_string());
    ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "no-cache")], body).into_response()
}

async fn index() -> Response {
    asset("index.html", include_str!("../static/index.html"), "text/html; charset=utf-8")
}
async fn app_js() -> Response {
    asset("app.js", include_str!("../static/app.js"), "text/javascript; charset=utf-8")
}
async fn style_css() -> Response {
    asset("style.css", include_str!("../static/style.css"), "text/css; charset=utf-8")
}

/// Hörprobe beim Wechsel der Stimme (muss mit app.js übereinstimmen).
const VOICE_SAMPLE: &str = "¡Hola! Vamos a practicar español.";

#[tokio::main]
async fn main() {
    let content = match Content::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    // `--audio`: alle Übungssätze vorab vertonen und beenden.
    if std::env::args().any(|a| a == "--audio") {
        let mut texts: Vec<String> = content.topics.iter().flat_map(|t| &t.items).map(full_sentence).collect();
        texts.push(VOICE_SAMPLE.to_string());
        texts.sort();
        texts.dedup();
        tts::generate_all(tts::Tts::from_env(), texts).await;
        return;
    }

    let progress_path = PathBuf::from(std::env::var("GRAMATICA_PROGRESS").unwrap_or_else(|_| "progress.json".into()));
    let progress = Progress::load(&progress_path);
    let seed = (srs::now() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;

    let app = Arc::new(AppState { content, progress: Mutex::new(progress), rng: Mutex::new(seed), tts: tts::Tts::from_env() });
    let router = Router::new()
        .route("/", get(index))
        .route("/app.js", get(app_js))
        .route("/style.css", get(style_css))
        .route("/api/overview", get(overview))
        .route("/api/session", get(session))
        .route("/api/check", post(check_answer))
        .route("/api/hint", post(hint))
        .route("/api/explain", post(explain))
        .route("/api/verbs", get(verb_list))
        .route("/api/sprint", post(sprint_done))
        .route("/api/goal", post(set_goal))
        .route("/api/tts", get(tts))
        .with_state(app);

    // Fester Port per PORT, sonst ab 4747 den ersten freien nehmen.
    let fixed: Option<u16> = std::env::var("PORT").ok().and_then(|p| p.parse().ok());
    let candidates: Vec<u16> = match fixed {
        Some(p) => vec![p],
        None => (4747..4767).collect(),
    };
    let mut bound = None;
    for port in candidates {
        if let Ok(l) = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await {
            bound = Some(l);
            break;
        }
    }
    let Some(listener) = bound else {
        eprintln!("Kein freier Port gefunden. Einen anderen mit z. B. PORT=5050 wählen.");
        std::process::exit(1);
    };
    let addr = listener.local_addr().expect("lokale Adresse");
    println!("¡Hola! Gramática läuft auf http://{addr}  (Fortschritt: {})", progress_path.display());
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            println!("\n¡Hasta luego!");
        })
        .await
        .unwrap();
}
