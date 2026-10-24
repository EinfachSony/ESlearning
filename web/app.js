"use strict";

// ---------- Helfer ----------

const $ = (sel, root = document) => root.querySelector(sel);
const app = $("#app");

const api = { get: (url) => window.gramaticaApi(url), post: (url, body) => window.gramaticaApi(url, body) };

function esc(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

function normalize(s) {
  return s.replace(/[¿?¡!.,;:"«»“”–-]/g, " ").toLowerCase().split(/\s+/).filter(Boolean).join(" ");
}
function stripAccents(s) {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "");
}

const settings = {
  get(key, fallback) {
    try {
      const v = localStorage.getItem("gramatica." + key);
      return v === null ? fallback : JSON.parse(v);
    } catch {
      return fallback;
    }
  },
  set(key, value) {
    try { localStorage.setItem("gramatica." + key, JSON.stringify(value)); } catch {}
  },
};

const pick = (arr) => arr[Math.floor(Math.random() * arr.length)];
const PRAISE = ["¡Muy bien!", "¡Perfecto!", "¡Genial!", "¡Excelente!", "¡Eso es!", "¡Fenomenal!", "¡Bravo!"];
const KIND_LABEL = { cloze: "Lücke füllen", choice: "Auswählen", order: "Satz bauen", translate: "Übersetzen" };

// ---------- Vorlesen (Web Speech API) ----------

let esVoice = null;
function loadVoice() {
  const voices = speechSynthesis.getVoices();
  esVoice = voices.find((v) => v.lang === "es-AR") || voices.find((v) => /^es-(UY|CL|PE|CO|MX|US)/i.test(v.lang)) || voices.find((v) => v.lang.startsWith("es")) || null;
}
if ("speechSynthesis" in window) {
  loadVoice();
  speechSynthesis.onvoiceschanged = loadVoice;
}
// Google-Stimme (vorab erzeugte Dateien in audio/), Browser-Stimme als Rückfall.
const VOICE_SAMPLE = "¡Hola! Vamos a practicar español."; // auch in main.rs, damit vorab vertont
settings.set("voice", "male");
let googleTts = true;
let audio = null;
const audioPool = new Map();
function primeSpeech(text) {
  if (!text || !googleTts || !window.gramaticaAudio(text) || audioPool.has(text)) return;
  const clip = new Audio(); clip.preload = "auto"; clip.src = window.gramaticaAudio(text);
  clip.load(); audioPool.set(text, clip);
}
function speak(text, rate=1) {
 const url=window.gramaticaAudio(text);if(!url){toast('Für diesen Satz fehlt noch die Aufnahme.');return;}
 const next=audioPool.get(text)||new Audio(url);if(audio===next&&!audio.paused){audio.playbackRate=rate;return;}
 if(audio)audio.pause();audio=next;audio.preload='auto';audio.currentTime=0;audio.playbackRate=rate;
 audio.onerror=()=>toast('Aufnahme konnte nicht geladen werden. Bitte erneut versuchen.');
 audio.play().catch(e=>{if(e.name!=='AbortError')toast('Tippe auf den Lautsprecher, um die Aufnahme zu starten.');});
}

// ---------- Kopfzeile ----------

let lastToday = null;
function updateHud(stats) {
  if ("google_tts" in stats) googleTts = stats.google_tts;
  $("#xp").textContent = stats.xp;
  $("#streak").textContent = stats.streak;
  $(".flame").classList.toggle("dim", stats.today_xp === 0);
  $("#goal-text").textContent = `${Math.min(stats.today_xp, 9999)}/${stats.daily_goal}`;
  const frac = Math.min(1, stats.today_xp / stats.daily_goal);
  $("#goal-ring").style.strokeDashoffset = String(97.4 * (1 - frac));
  if (lastToday !== null && lastToday < stats.daily_goal && stats.today_xp >= stats.daily_goal) {
    confetti(120);
    toast("🎯 Tagesziel erreicht! ¡Enhorabuena!");
  }
  lastToday = stats.today_xp;
}

function confetti(n = 80) {
  const box = $("#confetti");
  const colors = ["#c8452c", "#f2b134", "#2f8f5b", "#3b82f6", "#e46a4f"];
  for (let i = 0; i < n; i++) {
    const p = document.createElement("i");
    p.style.left = Math.random() * 100 + "vw";
    p.style.background = pick(colors);
    p.style.animationDuration = 1.6 + Math.random() * 1.6 + "s";
    p.style.animationDelay = Math.random() * 0.4 + "s";
    box.appendChild(p);
    setTimeout(() => p.remove(), 3800);
  }
}

function toast(text) {
  const t = document.createElement("div");
  t.className = "xp-float";
  t.style.left = "50%";
  t.style.top = "90px";
  t.style.transform = "translateX(-50%)";
  t.style.animationDuration = "2.6s";
  t.textContent = text;
  document.body.appendChild(t);
  setTimeout(() => t.remove(), 2700);
}

function floatXp(xp, anchor) {
  if (!xp) return;
  const r = anchor.getBoundingClientRect();
  const el = document.createElement("div");
  el.className = "xp-float";
  el.textContent = `+${xp} XP`;
  el.style.left = r.left + r.width / 2 - 30 + "px";
  el.style.top = r.top - 10 + "px";
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 1000);
}

// ---------- Übersicht ----------

let overview = null;

function stars(mastery) {
  const n = mastery >= 90 ? 3 : mastery >= 60 ? 2 : mastery >= 30 ? 1 : 0;
  return `<span class="stars">${[0, 1, 2].map((i) => `<span class="${i < n ? "" : "off"}">⭐</span>`).join("")}</span>`;
}

async function renderHome() {
  view = "home";
  overview = await api.get("/api/overview");
  const { stats, topics, recommended } = overview;
  updateHud(stats);
  if (!settings.get("voiceQualityMigrated", false)) { settings.set("voice", "male"); settings.set("voiceQualityMigrated", true); }
  $("#lvl").textContent = stats.level;
  const rec = topics.find((t) => t.id === recommended);
  const started = topics.some((t) => t.seen > 0);

  const game = await api.get("/api/game");
  applyTheme(game);
  app.innerHTML = gameHome(game) + `
    <section class="hero">
      <div class="card mix">
        <h2>${started ? "Täglicher Mix" : "¡Bienvenido! Los geht’s"}</h2>
        <div class="muted">${
          started
            ? "Fällige Wiederholungen und neuer Stoff, gut durchmischt. Das System merkt sich, was du kannst."
            : "Wir starten mit dem ersten Thema. Erst eine kurze Regelkarte, dann direkt anwenden."
        }</div>
        <div class="chips">
          ${stats.due_total ? `<span class="chip">🔁 ${stats.due_total} fällig</span>` : ""}
          ${rec ? `<span class="chip">✨ Neu: ${esc(rec.title)}</span>` : `<span class="chip">🏆 Alle Themen begonnen</span>`}
          <span class="chip">⏱ ca. 5 Min.</span>
        </div>
        <button class="btn big" id="start-mix">${started ? "Mix starten" : "Erste Lektion"} →</button>
      </div>
      <div class="card sprint">
        <h2>⚡ Trainingsarena</h2>
        <p class="muted small">60 Sekunden, so viele Verbformen wie möglich. Bis zu 60 XP pro Tag; danach trainierst du weiter für deinen Rekord.</p>
        <div class="chips"><span class="chip">Rekord: ${stats.sprint_best}</span></div>
        <button class="btn ghost" id="start-sprint">Sprint starten</button>
      </div>
    </section>

    <div class="section-title"><h2>Themen ${esc(stats.level)}</h2><span class="muted small">Wähle frei, oder folge dem markierten Vorschlag</span></div>
    <section class="topics">
      ${topics
        .map(
          (t, i) => `
        <button class="topic ${t.id === recommended ? "rec" : ""}" data-topic="${t.id}">
          <span class="num">${String(i + 1).padStart(2, "0")}</span>
          <h3>${esc(t.title)}</h3>
          <span class="es">${esc(t.title_es)}</span>
          <div class="badges">
            ${t.id === recommended ? `<span class="badge rec">Als Nächstes</span>` : ""}
            ${t.due ? `<span class="badge due">${t.due} fällig</span>` : ""}
            ${t.seen === 0 && t.id !== recommended ? `<span class="badge new">neu</span>` : ""}
          </div>
          <div class="row"><div class="bar" style="flex:1"><i style="width:${t.mastery}%"></i></div>${stars(t.mastery)}</div>
          <span class="muted small">${t.seen}/${t.total} gesehen · ${t.mastery}% sicher · 📖 ${(t.theory_done||[]).length}/${(t.theory_parts||[]).length}</span>
        </button>`
        )
        .join("")}
    </section>

    <p class="tip">Dein Lernfortschritt wird in diesem Browser gespeichert. Beim Löschen der Website-Daten geht er verloren. Mit einer Sicherung kannst du ihn auf ein anderes Gerät übertragen.</p><div class="actions"><button class="btn ghost" id="export-progress">Fortschritt sichern</button><label class="btn ghost">Sicherung laden<input id="import-progress" type="file" accept="application/json" hidden></label></div>
    <p class="tip">🇦🇷 Argentinisches Spanisch: vos & ustedes. Du hörst Tomás, eine männliche argentinische Neuralstimme. Alle Übungssätze sind als vollständige Aufnahmen vorbereitet und werden während der Frage vorgeladen.</p><div class="footer-settings">
      <label>Tagesziel
        <select id="goal">
          ${[50, 100, 150, 250, 400].map((g) => `<option value="${g}" ${g === stats.daily_goal ? "selected" : ""}>${g} XP</option>`).join("")}
        </select>
      </label>
      <label><input type="checkbox" id="autospeak" ${settings.get("autoSpeak", true) ? "checked" : ""}> Lösungen vorlesen</label>
      <label ${stats.google_tts ? "" : "hidden"}>Stimme
        <select id="voice">
          
          <option value="male" ${settings.get("voice", "male") === "male" ? "selected" : ""}>Tomás · Argentinien · Neural</option>
        </select>
      </label>
      <label><input type="checkbox" id="showde" ${settings.get("showDe", true) ? "checked" : ""}> Übersetzung anzeigen</label>
    </div>`;

  wireGame();
  $("#export-progress").onclick = () => {
    const blob = new Blob([localStorage.getItem("gramatica.progress.v1") || "{}"], {type:"application/json"});
    const a = document.createElement("a"); a.href=URL.createObjectURL(blob); a.download="gramatica-fortschritt.json"; a.click(); setTimeout(()=>URL.revokeObjectURL(a.href),1000);
  };
  $("#import-progress").onchange = async e => {
    try { await window.gramaticaImport(await e.target.files[0].text()); await renderHome(); toast("Fortschritt geladen"); }
    catch { toast("Die Sicherung ist ungültig. Der bisherige Fortschritt bleibt erhalten."); }
  };
  $("#start-mix").onclick = () => startSession(null);
  $("#start-sprint").onclick = renderSprint;
  app.querySelectorAll("[data-topic]").forEach((b) => (b.onclick = () => renderTopic(b.dataset.topic)));
  $("#goal").onchange = async (e) => updateHud(await api.post("/api/goal", { goal: Number(e.target.value) }));
  $("#autospeak").onchange = (e) => settings.set("autoSpeak", e.target.checked);
  $("#showde").onchange = (e) => settings.set("showDe", e.target.checked);
  $("#voice").onchange = (e) => {
    settings.set("voice", e.target.value);
    speak(VOICE_SAMPLE);
  };
}

// ---------- Regelkarte ----------

let ruleStep = 0;
function theoryParts(t){return t.theory_parts||[{title:'Grundlagen',html:t.explanation,key:'intro'}];}
function ruleCard(t,intro){
 const parts=theoryParts(t);ruleStep=Math.min(ruleStep,parts.length-1);const part=parts[ruleStep],done=(t.theory_done||[]).includes(part.key);
 return `<article class="card rule"><div class="rpg-eyebrow">📖 AKADEMIE · ${esc(t.title)}</div><h2>${esc(part.title)}</h2>
 <p class="muted">Teil ${ruleStep+1} von ${parts.length} · ${(t.theory_done||[]).length}/${parts.length} abgeschlossen</p>
 <div class="bar"><i style="width:${(ruleStep+1)/parts.length*100}%"></i></div>
 <div class="explanation table-wrap" style="margin:24px 0">${part.html}</div>
 <p class="theory-reward">${done?'✓ Belohnung bereits erhalten':'✨ Abschließen: +8 XP · +2 Gold'}${ruleStep===parts.length-1?' · Kapitelbonus: +15 XP und +5 Gold bei vollständigem Abschluss':''}</p>
 <div class="actions">${ruleStep?'<button class="btn ghost" id="rule-prev">← Zurück</button>':''}
 <button class="btn big" id="rule-complete">${done?(ruleStep<parts.length-1?'Nächster Teil →':'Übungen starten →'):'Gelesen · Belohnung erhalten →'}</button>
 <button class="btn ghost" id="back">Zur Übersicht</button></div></article>`;
}
function wireRule(t,intro,onBack){
 const draw=()=>{app.innerHTML=ruleCard(t,intro);wireRule(t,intro,onBack);window.scrollTo(0,0);};
 const prev=$('#rule-prev');if(prev)prev.onclick=()=>{ruleStep--;draw();};$('#back').onclick=onBack;
 $('#rule-complete').onclick=async()=>{
 const btn=$('#rule-complete');btn.disabled=true;
 try{const r=await api.post('/api/theory',{topic:t.id,part:theoryParts(t)[ruleStep].key});t.theory_done=r.done;updateHud(r.stats);
 if(r.xp_gained){toast(`+${r.xp_gained} XP · +${r.gold_gained} Gold${r.level_up?' · LEVEL '+r.player_level+'!':''}`);confetti(r.level_up?80:20);}
 if(ruleStep<theoryParts(t).length-1){ruleStep++;draw();}else{ruleStep=0;if(intro)renderQuestion();else startSession(t.id);}
 }catch(e){btn.disabled=false;toast('Speichern fehlgeschlagen. Bitte erneut versuchen.');}
 };
}

async function renderTopic(id) {
  view = "topic";
  if (!overview) overview = await api.get("/api/overview");
  const t = overview.topics.find((x) => x.id === id);
  ruleStep=Math.max(0,theoryParts(t).findIndex(x=>!(t.theory_done||[]).includes(x.key))); app.innerHTML = ruleCard(t, false); wireRule(t,false,renderHome);
  window.scrollTo(0, 0);
}

// ---------- Übungssitzung ----------

let view = "home";
let S = null; // Sitzungszustand

async function startSession(topic) {
  const data = await api.get("/api/session" + (topic ? `?topic=${encodeURIComponent(topic)}` : ""));
  overview = await api.get("/api/overview");
  updateHud(overview.stats);
  if (!data.items.length) {
    app.innerHTML = `<div class="card"><h2>Nichts zu tun 🎉</h2><p class="muted">Gerade ist nichts fällig. Komm später wieder oder wähle ein Thema.</p><button class="btn" id="back">Zur Übersicht</button></div>`;
    $("#back").onclick = renderHome;
    return;
  }
  S = {
    topic,
    queue: data.items.map((it) => ({ ...it, relearn: false, tries: 0 })),
    idx: 0,
    total: data.items.length,
    done: new Set(),
    firstTry: new Map(), // id -> erster Versuch richtig?
    mistakes: [],
    combo: 0,
    bestCombo: 0,
    xp: 0,
  };
  if (data.intro) {
    view = "intro";
    const t = overview.topics.find((x) => x.id === data.intro);
    ruleStep=Math.max(0,theoryParts(t).findIndex(x=>!(t.theory_done||[]).includes(x.key))); app.innerHTML = ruleCard(t, true); wireRule(t,true,renderHome);
  } else {
    renderQuestion();
  }
}

function promptHtml(it) {
 if(it.exerciseMode==='listen')return `<p class="muted">Höre zu und tippe den vollständigen spanischen Satz.</p><button class="btn ghost" id="question-audio">🔊 Satz anhören</button>`;
 if(it.exerciseMode==='gaps')return `<p class="muted">Ergänze mehrere fehlende Wörter. Tippe den vollständigen Satz.</p><div class="prompt">${esc(it.gapped)}</div><p class="muted">🇩🇪 ${esc(it.prompt)}</p>`;
 if(it.exerciseMode==='mission')return `<p class="muted">⚔️ Mini-Mission: Formuliere beide Sätze auf Spanisch.</p><div class="prompt">${esc(it.prompt)}</div>`;
  if (it.kind === "order" || it.kind === "translate") {
    return `<div class="muted small">${it.kind === "order" ? "Bilde den spanischen Satz:" : "Übersetze ins Spanische:"}</div>
            <div class="prompt">${esc(it.prompt)}</div>`;
  }
  const hint = it.hint ? ` <span class="hint-chip">(${esc(it.hint)})</span>` : "";
  const parts = esc(it.prompt).split("___");
  const body = parts.length > 1 ? parts[0] + `<span class="gap" id="gap">&nbsp;</span>` + hint + parts.slice(1).join("___") : parts[0] + hint;
  return `<div class="prompt">${body}</div>`;
}

function deLine(it) {
  if (!it.de || it.kind === "order" || it.kind === "translate") return `<div class="de-line"></div>`;
  return settings.get("showDe", true)
    ? `<div class="de-line">🇩🇪 ${esc(it.de)}</div>`
    : `<div class="de-line"><button class="de-toggle" id="de-toggle">Übersetzung zeigen</button></div>`;
}

const ACCENTS = ["á", "é", "í", "ó", "ú", "ñ", "ü", "¿", "¡"];
function accentBar() {
  return `<div class="accents">${ACCENTS.map((a) => `<button type="button" tabindex="-1" data-ch="${a}">${a}</button>`).join("")}</div>`;
}
function wireAccents(input) {
  app.querySelectorAll(".accents button").forEach(
    (b) =>
      (b.onmousedown = (e) => {
        e.preventDefault();
        const { selectionStart: s, selectionEnd: en, value } = input;
        input.value = value.slice(0, s) + b.dataset.ch + value.slice(en);
        input.selectionStart = input.selectionEnd = s + 1;
        input.dispatchEvent(new Event("input"));
      })
  );
}

function renderQuestion() {
  view = "practice";
  const it = S.queue[S.idx];
  S.phase = "ask";
  primeSpeech(it.audioText);
  S.hintUsed = false;
  S.selected = null;
  S.tray = [];

  let input = "";
  if (it.kind === "cloze" || it.kind === "translate") {
    input = `<input class="answer-input" id="answer" autocomplete="off" autocapitalize="off" spellcheck="false" placeholder="${
      it.kind === "cloze" ? "Fehlendes Wort …" : "Spanischer Satz …"
    }">${accentBar()}`;
  } else if (it.kind === "choice") {
    input = `<div class="options">${it.options
      .map((o, i) => `<button class="option" data-opt="${esc(o)}"><kbd>${i + 1}</kbd>${esc(o)}</button>`)
      .join("")}</div>`;
  } else {
    input = `<div class="tray" id="tray"></div>
             <div class="bank">${it.words.map((w, i) => `<button class="tile" data-i="${i}">${esc(w)}</button>`).join("")}</div>`;
  }

  const pct = (S.done.size / S.total) * 100;
  app.innerHTML = `
    <div class="practice-top">
      <button class="close" id="quit" title="Beenden">×</button>
      <div class="bar"><i style="width:${pct}%"></i></div>
      <div class="combo" id="combo">${S.combo >= 2 ? `🔥×${S.combo}` : ""}</div>
    </div>
    <article class="card" id="qcard">
      <div class="q-meta">
        <span class="q-type">${({translate:"Frei übersetzen",gaps:"Mehrere Lücken",listen:"Hörverständnis",mission:"Mini-Mission"})[it.exerciseMode]||KIND_LABEL[it.kind]}</span> · <span>${esc(it.topic_title)}</span>
        ${it.relearn ? `<span class="badge due">Nochmal</span>` : it.is_new ? `<span class="badge new">neu</span>` : ""}
      </div>
      ${promptHtml(it)}
      ${deLine(it)}
      ${input}
      <div id="feedback"></div>
      <div class="actions" id="actions">
        <button class="btn" id="submit">Prüfen</button>
        <button class="btn ghost" id="hint">💡 Tipp</button>
        <span class="spacer"></span>
        <span class="muted small" id="keyhint">Enter = prüfen</span>
      </div>
    </article>`;

  $("#quit").onclick = () => (S.done.size ? renderSummary() : renderHome());
  $("#hint").onclick = useHint;
  if($("#question-audio"))$("#question-audio").onclick=()=>speak(it.audioText);
  $("#submit").onclick = submit;
  const tog = $("#de-toggle");
  if (tog) tog.onclick = () => (tog.parentElement.textContent = "🇩🇪 " + it.de);

  if (it.kind === "cloze" || it.kind === "translate") {
    const inp = $("#answer");
    wireAccents(inp);
    const gap = $("#gap");
    inp.oninput = () => {
      if (gap && S.phase === "ask") gap.textContent = inp.value || " ";
    };
    inp.focus();
  } else if (it.kind === "choice") {
    $("#submit").hidden = true;
    $("#keyhint").textContent = "Tasten 1–" + it.options.length;
    app.querySelectorAll(".option").forEach((b) => (b.onclick = () => chooseOption(b)));
  } else {
    app.querySelectorAll(".tile").forEach((b) => (b.onclick = () => moveTile(b)));
  }
}

function chooseOption(btn) {
  if (S.phase !== "ask") return;
  S.selected = btn.dataset.opt;
  btn.classList.add("sel");
  const gap = $("#gap");
  if (gap) gap.textContent = S.selected;
  submit();
}

function moveTile(btn) {
  if (S.phase !== "ask" || btn.classList.contains("used")) return;
  const it = S.queue[S.idx];
  const i = Number(btn.dataset.i);
  btn.classList.add("used");
  S.tray.push(i);
  const chip = document.createElement("button");
  chip.className = "tile";
  chip.textContent = it.words[i];
  chip.onclick = () => {
    if (S.phase !== "ask") return;
    S.tray.splice(S.tray.indexOf(i), 1);
    chip.remove();
    btn.classList.remove("used");
  };
  $("#tray").appendChild(chip);
}

function currentAnswer(it) {
  if (it.kind === "choice") return S.selected ?? "";
  if (it.kind === "order") return S.tray.map((i) => it.words[i]).join(" ");
  return $("#answer").value;
}

async function useHint() {
  if (S.phase !== "ask" || S.hintUsed) return;
  const it = S.queue[S.idx];
  S.hintUsed = true;
  $("#hint").disabled = true;
  const h = await api.post("/api/hint", { id: it.id });
  if (h.remove) {
    app.querySelectorAll(".option").forEach((b) => b.dataset.opt === h.remove && b.classList.add("gone"));
  } else if (h.text) {
    const inp = $("#answer");
    if (inp) inp.placeholder = h.text;
    else $("#keyhint").textContent = "Beginnt mit: " + h.text;
  }
}

async function submit() {
  if (S.phase !== "ask") return;
  const it = S.queue[S.idx];
  const answer = currentAnswer(it).trim();
  if (!answer) {
    $("#qcard").classList.remove("shake");
    void $("#qcard").offsetWidth;
    $("#qcard").classList.add("shake");
    return;
  }
  S.phase = "checking";
  const correctNow = (v) => v !== "wrong";
  const res = await api.post("/api/check", {
    id: it.id,
    answer,
    hint_used: S.hintUsed,
    relearn: it.relearn,
    combo: S.combo,
  });

  // Statistik der Sitzung
  if (!S.firstTry.has(it.id)) S.firstTry.set(it.id, correctNow(res.verdict) && !S.hintUsed);
  if (correctNow(res.verdict) && !S.hintUsed) {
    S.combo++;
    S.bestCombo = Math.max(S.bestCombo, S.combo);
  } else {
    S.combo = 0;
  }
  S.xp += res.xp_gained;
  updateHud(res.stats);
  const combo = $("#combo");
  combo.textContent = S.combo >= 2 ? `🔥×${S.combo}` : "";
  if (S.combo >= 2) {
    combo.classList.add("pop");
    setTimeout(() => combo.classList.remove("pop"), 200);
  }

  if (res.verdict === "wrong") {
    S.mistakes.push({ prompt: it.kind === "order" || it.kind === "translate" ? it.prompt : it.prompt.replace("___", "…"), yours: answer, right: res.sentence, explain: res.explain, notes: res.notes || [], topic: it.topic });
    // Erneut abfragen, ein paar Karten später (Wiederholung, bis es sitzt).
    if (it.tries < 2) {
      const again = { ...it, relearn: true, tries: it.tries + 1 };
      S.queue.splice(Math.min(S.idx + 3, S.queue.length), 0, again);
    } else {
      S.done.add(it.id);
    }
  } else {
    S.done.add(it.id);
  }

  showFeedback(it, res, answer);
  if(res.events?.length) { toast(res.events.join(" · ")); if(res.events.some(x=>x.startsWith("Level"))) confetti(65); }
}

// Vollständige Regel der Lektion; bei falscher Antwort aufgeklappt.
function ruleBox(topicId, open) {
  const t = overview?.topics.find((x) => x.id === topicId);
  if (!t) return "";
  return `
    <details class="rule-box" ${open ? "open" : ""}>
      <summary>📘 Regel nachlesen: ${esc(t.title)}</summary>
      <div class="explanation table-wrap">${t.explanation}${t.details || ""}</div>
    </details>`;
}

function showFeedback(it, res, answer) {
  const v = res.verdict;
  const cls = v === "correct" ? "ok" : v === "accent" ? "warn" : "bad";

  // Eingabe einfärben
  if (it.kind === "choice") {
    app.querySelectorAll(".option").forEach((b) => {
      b.disabled = true;
      if (normalize(b.dataset.opt) === normalize(res.expected)) b.classList.add("ok");
      else if (b.dataset.opt === answer) b.classList.add("bad");
    });
  } else {
    const inp = $("#answer");
    if (inp) {
      inp.classList.add(cls);
      inp.readOnly = true;
    }
  }
  const gap = $("#gap");
  if (gap) gap.textContent = res.expected;

  const title = v === "correct" ? pick(PRAISE) : v === "accent" ? "¡Casi! Achte auf die Akzente" : "Noch nicht ganz";
  const needRetype = v === "wrong" && (it.kind === "cloze" || it.kind === "translate");
  $("#feedback").innerHTML = `
    <div class="feedback ${cls}">
      <h3>${title}</h3>
      ${v !== "correct" ? `<div class="solution">${esc(res.expected)}</div>` : ""}
      ${
        res.notes?.length
          ? `<div class="why"><b>${v === "wrong" ? "Warum falsch?" : "Hinweis"}</b><ul>${res.notes.map((n) => `<li>${esc(n)}</li>`).join("")}</ul></div>`
          : ""
      }
      <div>${esc(res.sentence)} <button class="speak" id="speak" title="Vorlesen">🔊</button></div>
      ${res.de && (it.kind === "order" || it.kind === "translate" || !settings.get("showDe", true)) ? `<div class="muted small">🇩🇪 ${esc(res.de)}</div>` : ""}
      ${res.explain ? `<p class="explain small">${v === "correct" ? "✓ " : "📘 Regel: "}${esc(res.explain)}</p>` : ""}
      ${needRetype ? `<input class="answer-input" id="retype" autocomplete="off" spellcheck="false" placeholder="Zum Einprägen: Lösung abtippen …">${accentBar()}` : ""}
      ${v !== "correct" ? ruleBox(it.topic, v === "wrong") : ""}
    </div>`;
  $("#speak").onclick = () => speak(res.sentence);
  if (settings.get("autoSpeak", true)) speak(res.sentence);

  floatXp(res.xp_gained, $("#submit").hidden ? $("#feedback") : $("#submit"));

  $("#actions").innerHTML = `
    <button class="btn ${v === "wrong" ? "" : "ok"}" id="next">Weiter →</button>
    ${needRetype ? `<button class="link" id="skip">überspringen</button>` : ""}
    <span class="spacer"></span><span class="muted small">Enter = weiter</span>`;
  $("#next").onclick = next;

  if (needRetype) {
    S.phase = "retype";
    const r = $("#retype");
    wireAccents(r);
    $("#next").disabled = true;
    r.oninput = () => {
      const ok = stripAccents(normalize(r.value)) === stripAccents(normalize(res.expected));
      $("#next").disabled = !ok;
      r.classList.toggle("ok", ok);
    };
    $("#skip").onclick = next;
    r.focus();
  } else {
    S.phase = "done";
    $("#next").focus();
  }
}

function next() {
  S.idx++;
  if (S.idx >= S.queue.length) renderSummary();
  else renderQuestion();
}

function renderSummary() {
  view = "summary";
  const n = S.firstTry.size;
  const right = [...S.firstTry.values()].filter(Boolean).length;
  const acc = n ? Math.round((right / n) * 100) : 0;
  const emoji = acc >= 90 ? "🏆" : acc >= 70 ? "🎉" : acc >= 50 ? "💪" : "🌱";
  const msg =
    acc >= 90 ? "¡Increíble! Das sitzt." : acc >= 70 ? "¡Muy bien! Starke Runde." : acc >= 50 ? "Gute Arbeit, Fehler sind Lernchancen." : "Jeder Fehler hilft beim Lernen. Die schwierigen Übungen kommen bald wieder.";
  if (acc >= 80) confetti(90);
  const errTopics = [...new Set(S.mistakes.map((m) => m.topic))].map((id) => overview.topics.find((t) => t.id === id)).filter(Boolean);

  app.innerHTML = `
    <article class="card">
      <div class="quest-note">⚔️ Expedition abgeschlossen. Deine Quests und Belohnungen warten in der Übersicht.</div>
      <div class="summary-head">
        <div class="big-emoji">${emoji}</div>
        <h2>${msg}</h2>
      </div>
      <div class="kpis">
        <div class="kpi"><b>${acc}%</b><span>beim ersten Versuch</span></div>
        <div class="kpi"><b>+${S.xp}</b><span>XP</span></div>
        <div class="kpi"><b>${S.bestCombo}</b><span>beste Serie</span></div>
        <div class="kpi"><b>${n}</b><span>Übungen</span></div>
      </div>
      ${
        S.mistakes.length
          ? `<h3 style="margin-top:12px">Zum Merken</h3>
             <ul class="mistakes">${S.mistakes
               .map(
                 (m) => `<li><div class="muted small">${esc(m.prompt)}</div>
                         <span class="yours">${esc(m.yours)}</span> → <span class="right">${esc(m.right)}</span>
                         ${m.notes.map((n) => `<div class="small">→ ${esc(n)}</div>`).join("")}
                         ${m.explain ? `<div class="small muted">📘 ${esc(m.explain)}</div>` : ""}</li>`
               )
               .join("")}</ul>`
          : `<p class="muted" style="text-align:center">Keine Fehler, perfekte Runde!</p>`
      }
      ${
        errTopics.length
          ? `<h3 style="margin-top:18px">Regeln wiederholen</h3>
             <div class="row" style="margin-top:8px">${errTopics
               .map((t) => `<button class="btn ghost" data-rule="${t.id}">📘 ${esc(t.title)}</button>`)
               .join("")}</div>`
          : ""
      }
      <div class="actions" style="justify-content:center">
        <button class="btn big" id="again">Weiter üben</button>
        <button class="btn ghost" id="home">Übersicht</button>
      </div>
      <p class="tip" style="text-align:center">Kurze, regelmäßige Einheiten wirken besser als lange Sitzungen. Morgen wiederholt das System, was fällig ist.</p>
    </article>`;
  $("#again").onclick = () => startSession(S.topic);
  $("#home").onclick = renderHome;
  app.querySelectorAll("[data-rule]").forEach((b) => (b.onclick = () => renderTopic(b.dataset.rule)));
  $("#again").focus();
}

// ---------- Verb-Sprint ----------

const PERSONS = ["yo", "vos", "él / ella / usted", "nosotros", "ustedes", "ellos / ellas"];
const GROUPS = ["regelmäßig", "Stammwechsel", "yo-unregelmäßig", "unregelmäßig"];
let verbs = null;
let SP = null;

async function renderSprint() {
  view = "sprint-setup";
  if (!verbs) verbs = await api.get("/api/verbs");
  const chosen = settings.get("sprintGroups", GROUPS);
  app.innerHTML = `
    <article class="card sprint-stage">
      <h2>⚡ Trainingsarena</h2>
      <p class="muted">Konjugiere im Präsens, so schnell du kannst. Richtige Formen bringen Punkte, Fehler kosten nur Zeit.</p>
      <div class="row" style="justify-content:center;margin:14px 0">
        ${GROUPS.map(
          (g) => `<label class="chip"><input type="checkbox" value="${g}" ${chosen.includes(g) ? "checked" : ""}> ${g} (${verbs.filter((v) => v.group === g).length})</label>`
        ).join("")}
      </div>
      <div class="actions" style="justify-content:center">
        <button class="btn big" id="go">Start (60 s)</button>
        <button class="btn ghost" id="back">Zurück</button>
      </div>
    </article>`;
  $("#back").onclick = renderHome;
  $("#go").onclick = () => {
    const groups = [...app.querySelectorAll("input[type=checkbox]:checked")].map((c) => c.value);
    if (!groups.length) return;
    settings.set("sprintGroups", groups);
    startSprint(verbs.filter((v) => groups.includes(v.group)));
  };
}

function startSprint(pool) {
  view = "sprint";
  SP = { pool, score: 0, left: 60, cur: null, misses: [], timer: null, last: null };
  app.innerHTML = `
    <article class="card sprint-stage">
      <div class="timer" id="timer">60</div>
      <div class="muted">Punkte: <b id="score">0</b></div>
      <div class="verb-prompt" id="vp"></div>
      <div class="verb-de" id="vde"></div>
      <input class="answer-input" id="sp-in" autocomplete="off" autocapitalize="off" spellcheck="false" placeholder="Form eintippen + Enter">
      ${accentBar()}
      <div class="sprint-flash" id="flash"></div>
    </article>`;
  wireAccents($("#sp-in"));
  nextVerb();
  $("#sp-in").focus();
  SP.timer = setInterval(() => {
    SP.left--;
    $("#timer").textContent = SP.left;
    $("#timer").classList.toggle("low", SP.left <= 10);
    if (SP.left <= 0) endSprint();
  }, 1000);
}

function nextVerb() {
  let v, p;
  do {
    v = pick(SP.pool);
    p = Math.floor(Math.random() * 6);
  } while (SP.pool.length > 1 && SP.last === v.inf);
  SP.last = v.inf;
  SP.cur = { v, p };
  $("#vp").innerHTML = `<span class="person">${PERSONS[p]}</span> · ${esc(v.inf)}`;
  $("#vde").textContent = v.de;
  $("#sp-in").value = "";
}

function sprintAnswer() {
  const inp = $("#sp-in");
  const given = normalize(inp.value);
  if (!given) return;
  const { v, p } = SP.cur;
  const want = v.forms[p];
  const flash = $("#flash");
  if (given === normalize(want)) {
    SP.score++;
    flash.className = "sprint-flash ok";
    flash.textContent = "✓ " + want;
  } else if (stripAccents(given) === stripAccents(normalize(want))) {
    SP.score++;
    flash.className = "sprint-flash ok";
    flash.textContent = "✓ (Akzent: " + want + ")";
  } else {
    flash.className = "sprint-flash bad";
    flash.textContent = `✗ ${PERSONS[p]} ${want}`;
    const miss = `${v.inf} · ${PERSONS[p]} → ${want}`;
    SP.misses.push(miss);
    // Grund nachladen und anzeigen (bleibt auch in der Auswertung)
    const idx = SP.misses.length - 1;
    const text = flash.textContent;
    api.post("/api/explain", { given: inp.value, expected: want }).then((notes) => {
      if (!notes.length || !SP) return;
      SP.misses[idx] = `${miss} – ${notes[0]}`;
      if (flash.textContent === text) flash.textContent = `${text} – ${notes[0]}`;
    });
  }
  $("#score").textContent = SP.score;
  nextVerb();
}

async function endSprint() {
  clearInterval(SP.timer);
  view = "sprint-end";
  const res = await api.post("/api/sprint", { score: SP.score });
  updateHud(res.stats);
  if (res.new_record) confetti(120);
  app.innerHTML = `
    <article class="card sprint-stage">
      <div class="big-emoji" style="font-size:3rem">${res.new_record ? "🏆" : "⚡"}</div>
      <h2>${SP.score} Formen${res.new_record ? " – neuer Rekord!" : ""}</h2>
      <p class="muted">Rekord: ${res.best} · +${res.xp_gained} XP</p>
      ${
        SP.misses.length
          ? `<h3>Zum Merken</h3><ul class="mistakes" style="text-align:left">${[...new Set(SP.misses)].map((m) => `<li>${esc(m)}</li>`).join("")}</ul>`
          : ""
      }
      <div class="actions" style="justify-content:center">
        <button class="btn big" id="again">Nochmal</button>
        <button class="btn ghost" id="home">Übersicht</button>
      </div>
    </article>`;
  $("#again").onclick = () => startSprint(SP.pool);
  $("#home").onclick = renderHome;
  $("#again").focus();
}

// ---------- Tastatur ----------

document.addEventListener("keydown", (e) => {
  if (view === "practice" && S) {
    const it = S.queue[S.idx];
    if (e.key === "Enter") {
      e.preventDefault();
      if (S.phase === "ask") submit();
      else if (S.phase === "done") next();
      else if (S.phase === "retype" && !$("#next").disabled) next();
    } else if (S.phase === "ask" && it.kind === "choice" && /^[1-9]$/.test(e.key)) {
      const btn = app.querySelectorAll(".option")[Number(e.key) - 1];
      if (btn && !btn.classList.contains("gone")) chooseOption(btn);
    } else if (S.phase === "ask" && it.kind === "order" && e.key === "Backspace") {
      const last = $("#tray").lastElementChild;
      if (last) last.click();
    }
  } else if (view === "sprint" && e.key === "Enter") {
    e.preventDefault();
    sprintAnswer();
  } else if (view === "intro" && e.key === "Enter") {
    e.preventDefault();
    renderQuestion();
  }
});

document.querySelector("[data-go=home]").onclick = () => {
  if (SP?.timer) clearInterval(SP.timer);
  renderHome();
};

// Direkteinstieg per Lesezeichen: #mix, #sprint, #topic=ser
await window.gramaticaReady;
applyTheme(await api.get("/api/game"));
const hash = location.hash.slice(1);
if (hash === "adventure") openAdventure();
else if (hash === "mix") startSession(null);
else if (hash === "sprint") renderSprint();
else if (hash.startsWith("topic=")) renderTopic(hash.slice(6));
else renderHome();

function applyTheme(g) {
 document.documentElement.dataset.realm=g.theme;
 const color=g.rewards.find(r=>r.id===g.theme)?.color||"#e79b52";
 document.documentElement.style.setProperty("--realm",color);
}
function gameHome(g) {
 const percent=Math.min(100,Math.round((g.xp-g.floor)/(g.next-g.floor)*100));
 return `<section class="rpg-hero"><div class="rpg-eyebrow">DEINE SPRACHREISE · KAMPAGNE A1 · ARGENTINIEN</div><div class="rpg-character"><div class="rpg-avatar">${g.icon}</div><div><span class="rpg-kicker">SONY · ${esc(g.title)}</span><h1>Dein nächstes Abenteuer<br>beginnt mit einem Satz.</h1></div><div class="rpg-level"><b>${g.level}</b><span>LEVEL</span></div></div><div class="rpg-xp"><div><b>${g.xp-g.floor} / ${g.next-g.floor} XP</b><span> bis Level ${g.level+1}</span></div><div class="rpg-track"><i style="width:${percent}%"></i></div></div><button class="btn" id="rpg-start" style="margin-top:18px">🗺️ Südamerika-Expedition →</button><div class="rpg-stats"><span>🪙 <b>${g.gold}</b> Gold</span><span>🔮 <b>${g.mastered}/${g.total}</b> langfristig gefestigt</span><span>🏕️ <b>${g.days}</b> Lerntage</span></div></section>
 <section class="adventure-invite"><h2>🗺️ Deine Minigames warten</h2><p>Sechs Reiseorte, Gespräche mit Entscheidungen, Bosskämpfe, Funkmissionen, Satzschmiede, Lagerverteidigung und Magierprüfungen.</p><button class="btn ghost" id="adventure-launch">Karte, Talente & Sammlung öffnen →</button></section>
 <section class="rpg-section"><div class="rpg-heading"><h2>Dein Questlog</h2><span>Jeden Tag neue Ziele · kein Verlust bei Pausen</span></div><div class="rpg-quests">${g.quests.map(q=>`<article class="rpg-quest"><div class="quest-icon">${q.icon}</div><h3>${q.name}</h3><p>${q.desc}</p><div class="rpg-track"><i style="width:${q.count/q.goal*100}%"></i></div><div class="quest-bottom"><span>${q.count}/${q.goal} · 🪙 ${q.reward}</span><button class="btn ghost" data-claim="${q.id}" ${q.claimed||q.count<q.goal?'disabled':''}>${q.claimed?'Abgeholt':q.count>=q.goal?'Belohnung holen':'In Arbeit'}</button></div></article>`).join('')}</div></section>
 <details class="rpg-camp"><summary>🏕️ Lager öffnen <span>Erfolge, Titel & Belohnungen</span></summary><h2>Deine Trophäen</h2><div class="rpg-trophies">${g.achievements.map(a=>`<div class="rpg-trophy ${a.unlocked?'earned':''}"><span>${a.icon}</span><b>${a.name}</b><small>${a.desc}</small><em>${a.unlocked?'Freigeschaltet':'🪙 '+a.gold+' Gold'}</em></div>`).join('')}</div><h2>Wähle dein Reich</h2><p class="muted">Gold schaltet dauerhafte Designs frei. Keine Zufallsbelohnungen und kein Echtgeld.</p><div class="rpg-shop">${g.rewards.map(r=>`<button class="rpg-reward" data-theme="${r.id}" ${!g.owned.includes(r.id)&&g.gold<r.price?'disabled':''}><i style="background:${r.color}"></i><b>${r.name}</b><small>${r.desc}</small><span>${g.theme===r.id?'✓ Ausgerüstet':g.owned.includes(r.id)?'Ausrüsten':'🪙 '+r.price}</span></button>`).join('')}</div><p class="rpg-note">Titel wachsen mit deinem Level: Reisender → Wortkundiger (5) → Sprachmagier (10) → Weltenwanderer (20) → Meister der Worte (35) → Leyenda (50).</p></details>
 <p class="rpg-note">Spielregel: XP gibt es für verschiedene gelöste Aufgaben. Wiederholtes Farmen derselben Aufgabe bringt keine weiteren Tages-XP. Wissen steigt erst bei fälligen Wiederholungen in die nächste Box. Nach einer Runde: kurz aufstehen, dann bewusst weiterlernen.</p>`;
}
function wireGame(){
 $("#rpg-start").onclick=()=>openAdventure();
 const launch=$("#adventure-launch");if(launch)launch.onclick=()=>openAdventure();
 app.querySelectorAll('[data-claim]').forEach(b=>b.onclick=async()=>{b.disabled=true;await api.post('/api/game-action',{action:'claim',id:b.dataset.claim});await renderHome();toast('Questbelohnung erhalten!');});
 app.querySelectorAll('[data-theme]').forEach(b=>b.onclick=async()=>{b.disabled=true;await api.post('/api/game-action',{action:'theme',id:b.dataset.theme});await renderHome();app.querySelector('.rpg-camp').open=true;});
}

async function openAdventure(id,topic){view="adventure";if(SP?.timer)clearInterval(SP.timer);await window.AdventureUI.open({app,api,esc,speak,primeSpeech,updateHud,toast,confetti,onHome:renderHome},{id,topic});}
