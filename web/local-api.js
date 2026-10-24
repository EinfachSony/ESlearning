(() => {
'use strict';
let content,verbs,audioMap,adventureData;
const key='gramatica.progress.v1';
let p; try {p=JSON.parse(localStorage.getItem(key));}catch{}
p=Object.assign({items:{},xp:0,daily:{},daily_goal:150,sprint_best:0},p||{});
const day=(offset=0)=>{const d=new Date();d.setDate(d.getDate()-offset);return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;};
const now=()=>Date.now()/1000;
function save(){try{localStorage.setItem(key,JSON.stringify(p));}catch{alert('Dein Fortschritt konnte im Browser nicht gespeichert werden. Bitte erlaube Website-Speicher.');}}
function addXP(x){p.xp+=x;p.daily[day()]=(p.daily[day()]||0)+x;save();}
const norm=s=>s.replace(/[¿?¡!.,;:"«»“”–-]/g,' ').replace(/’/g,"'").toLowerCase().split(/\s+/).filter(Boolean).join(' ');
const strip=s=>s.normalize('NFD').replace(/[\u0300-\u036f]/g,'');
const shuffle=a=>{a=[...a];for(let i=a.length-1;i>0;i--){let j=Math.floor(Math.random()*(i+1));[a[i],a[j]]=[a[j],a[i]];}return a;};
const all=()=>content.topics.flatMap(t=>t.items.map(i=>({t,i})));
const rec=()=>content.topics.find(t=>t.items.some(i=>!p.items[i.id]));
function stats(){let n=p.daily[day()]?0:1,s=0;while(p.daily[day(n)]>0){s++;n++;}return {xp:p.xp,today_xp:p.daily[day()]||0,daily_goal:p.daily_goal,streak:s,sprint_best:p.sprint_best,due_total:Object.values(p.items).filter(s=>s.due<=now()).length,level:content.level,google_tts:true};}
function overview(){p.theory ||= {};return {stats:stats(),recommended:rec()?.id,topics:content.topics.map(t=>({...t,theory_done:p.theory[t.id]||[],total:t.items.length,seen:t.items.filter(i=>p.items[i.id]).length,due:t.items.filter(i=>p.items[i.id]?.due<=now()).length,mastery:Math.floor(t.items.reduce((n,i)=>n+Math.min(4,p.items[i.id]?.box||0),0)*100/(t.items.length*4))}))};}
const sentence=i=>['cloze','choice'].includes(i.type)&&i.prompt.includes('___')?i.prompt.replace('___',i.answers[0]).replaceAll(' → ', ', ').replace(/ \([^)]*\)/g,''):i.answers[0];
function practiceItem(i){
 let q=['cloze','choice'].includes(i.type)&&i.prompt.includes('___')?{...i,type:'translate',prompt:i.de||i.prompt,answers:[sentence(i)]}:{...i,type:'translate'};
 if(!q.exerciseMode){const hash=[...i.id].reduce((n,c)=>n+c.charCodeAt(0),0);q.exerciseMode=q.answers[0].split(/\s+/).length<4?'translate':['translate','gaps','listen'][hash%3];}
 if(q.exerciseMode==='gaps'){let words=q.answers[0].split(/\s+/);const hide=new Set([0,Math.floor(words.length/2),words.length-1]);q.gapped=words.map((w,k)=>hide.has(k)?'____':w).join(' ');}
 q.answers=[...new Set(q.answers.flatMap(a=>[a,a.replace(/(^|[.!?]\s+)(yo|vos|nosotros|nosotras|ustedes)\s+(?!y\b)/gi,'$1')]))];
 return q;
}
function session(topic){const focus=topic?content.topics.find(t=>t.id===topic):rec();const pool=all().filter(x=>!topic||x.t.id===topic);const fresh=focus?focus.items.filter(i=>!p.items[i.id]).map(i=>({t:focus,i})):[];const due=pool.filter(x=>p.items[x.i.id]?.due<=now()).sort((a,b)=>p.items[a.i.id].due-p.items[b.i.id].due).slice(0,15-Math.min(fresh.length,topic?7:4));const news=fresh.slice(0,Math.min(15-due.length,topic?15:10));const weak=pool.filter(x=>p.items[x.i.id]?.due>now()).sort((a,b)=>p.items[a.i.id].box-p.items[b.i.id].box||p.items[a.i.id].due-p.items[b.i.id].due).slice(0,15-due.length-news.length);const reviews=shuffle([...due,...weak]);const rest=news.slice(2);const out=news.slice(0,2);while(rest.length||reviews.length){out.push(Math.random()*(rest.length+reviews.length)<rest.length?rest.shift():reviews.shift());}return {intro:focus&&news.length&&!focus.items.some(i=>p.items[i.id])&&!(focus.theory_parts||[]).every(x=>(p.theory?.[focus.id]||[]).includes(x.key))?focus.id:null,items:out.map(({t,i})=>{const q=practiceItem(i);return {...q,audioText:q.answers[0],kind:q.type,topic:t.id,topic_title:t.title,options:[],words:[],is_new:!p.items[i.id],level:p.items[i.id]?.box||0};})};}
const persons=['yo','vos','él/ella/usted','nosotros','ustedes','ellos/ellas'];
function explain(given,expected){const g=norm(given),e=norm(expected);if(strip(g)===strip(e)&&g!==e)return [`Achte auf die genaue Schreibweise: ${expected}. Akzente und ñ gehören zum Wort.`];for(const v of verbs){const target=v.forms.indexOf(e);if(target<0)continue;const actual=v.forms.indexOf(g);if(actual>=0)return [`${given} gehört zu ${persons[actual]}. Hier brauchst du ${persons[target]}: ${expected}.`];const stem=v.inf.slice(0,-2);const endings=v.inf.endsWith('ar')?['o','as','a','amos','áis','an']:v.inf.endsWith('er')?['o','es','e','emos','éis','en']:['o','es','e','imos','ís','en'];if(g===stem+endings[target]&&e!==g)return [`${v.inf} ist hier unregelmäßig. Die richtige Form für ${persons[target]} ist ${expected}.`];}
if(e.includes(' ')){const gs=g.split(' '),es=e.split(' '),missing=[...es],extra=[];for(const w of gs){let k=missing.indexOf(w);if(k<0)extra.push(w);else missing.splice(k,1);}if(!missing.length&&!extra.length)return ['Die Wörter stimmen, aber die Reihenfolge ist falsch. Vergleiche deinen Satz mit der Lösung.'];return [missing.length?'Fehlend oder anders benötigt: '+missing.join(', '):'',extra.length?'Überprüfe diese Wörter: '+extra.join(', '):''].filter(Boolean);}return [`Benötigt wird „${expected}“. Lies die Regel unten und vergleiche sie mit „${given}“.`];}
function check(b){const original=all().find(x=>x.i.id===b.id)?.i;if(!original)throw Error('Unbekannte Übung');const i=practiceItem(original);let verdict=i.answers.some(a=>norm(a)===norm(b.answer))?'correct':i.answers.some(a=>strip(norm(a))===strip(norm(b.answer)))?'accent':'wrong';const critical=new Set(['esta','está','estas','estás','este','esté','el','él','tu','tú','mi','mí','si','sí','se','sé','te','té','de','dé','mas','más','como','cómo','que','qué','quien','quién','donde','dónde','cuando','cuándo','cuanto','cuánto']);if(verdict==='accent'&&norm(b.answer).split(' ').some((w,k)=>critical.has(w)&&w!==norm(i.answers.find(a=>strip(norm(a))===strip(norm(b.answer)))||i.answers[0]).split(' ')[k]))verdict='wrong';
const wasNew=!p.items[i.id], wasDue=!!p.items[i.id]&&p.items[i.id].due<=now(), oldLevel=window.RPG.level(p.xp);
const oldBox=p.items[i.id]?.box||0, lastCorrect=p.items[i.id]?.lastCorrect||0;
const st=p.items[i.id]||{box:0,due:0,seen:0,correct:0,lapses:0};st.seen++;let xp=0;if(verdict==='wrong'){st.lapses++;st.box=st.box>=3?2:0;st.due=now()+60;}else{st.correct++;if(b.relearn||b.hint_used){st.due=now()+600;xp=b.relearn?3:4;}else{if(wasNew||wasDue)st.box=Math.min(7,st.box+1);st.due=now()+[60,600,86400,259200,604800,1382400,3024000,7776000][st.box];xp=(verdict==='correct'?10:6)+Math.min(10,b.combo||0);}}p.items[i.id]=st;
const eligible=verdict!=='wrong'&&!b.hint_used&&!b.relearn;
p.audioLearning ||= {hard:[],retained:[],claims:[]};const al=p.audioLearning;al.hard ||= [];al.retained ||= [];al.claims ||= [];let rare=false,rareReasons=[],mastered=eligible&&oldBox<4&&st.box>=4;
if(eligible){if(lastCorrect&&now()-lastCorrect>=604800&&!al.retained.includes(i.id))al.retained.push(i.id);st.lastCorrect=now();if(i.exerciseMode==='mission'&&!al.hard.includes(i.id))al.hard.push(i.id);const milestones=[['hard50',al.hard.length>=50],['retained7',al.retained.length>=1],['master100',Object.values(p.items).filter(x=>x.box>=4).length>=100],...content.topics.map(t=>['topic:'+t.id,t.items.every(q=>(p.items[q.id]?.box||0)>=4)])];for(const [id,ready] of milestones)if(ready&&!al.claims.includes(id)){al.claims.push(id);rare=true;rareReasons.push(id==='hard50'?'50 verschiedene Zwei-Satz-Missionen gelöst':id==='retained7'?'Nach mindestens sieben Tagen erneut richtig erinnert':id==='master100'?'100 verschiedene Übungen langfristig gefestigt':'Alle Übungen eines Grammatikgebiets langfristig gefestigt');}}
const daily=window.RPG.ensure(p).days[day()]||{};
daily.xpClaims ||= [];if(daily.xpClaims.includes(i.id)||(daily.practice||[]).includes(i.id))xp=0;else if(xp>0)daily.xpClaims.push(i.id);window.RPG.ensure(p).days[day()]=daily;
addXP(xp);
const events=window.RPG.record(p,content,day(),i.id,eligible,wasNew,wasDue);for(const reason of rareReasons)events.push('Lernmeilenstein: '+reason);save();
if(window.RPG.level(p.xp)>oldLevel)events.unshift(`Level ${window.RPG.level(p.xp)} erreicht!`);const why=Object.entries(i.why||{}).find(([k])=>norm(k)===norm(b.answer))?.[1];return {audio:{mastered,rare,hard:i.exerciseMode==='mission'},events,verdict,expected:i.answers[0],sentence:sentence(i),explain:i.explain,notes:verdict==='correct'?[]:why?[why]:explain(b.answer,i.answers[0]),de:i.de||(['translate','order'].includes(i.type)?i.prompt:null),xp_gained:xp,level:st.box,stats:stats()};}
window.gramaticaImport=text=>{const x=JSON.parse(text);if(!x||typeof x!=='object'||!x.items||typeof x.items!=='object'||Array.isArray(x.items)||!Number.isSafeInteger(x.xp)||x.xp<0||!x.daily||!Number.isFinite(x.daily_goal)||!Number.isFinite(x.sprint_best))throw Error('invalid');for(const st of Object.values(x.items)){if(!st||!Number.isInteger(st.box)||st.box<0||st.box>7||!Number.isFinite(st.due))throw Error('invalid');}for(const v of Object.values(x.daily)){if(!Number.isFinite(v)||v<0)throw Error('invalid');}p=x;save();};
window.gramaticaReady=Promise.all(['content.json','verbs.json','audio-map.json','adventure.json'].map(u=>fetch(u+'?v=20261024-adventure3').then(r=>{if(!r.ok)throw Error('Lerninhalte nicht geladen');return r.json();}))).then(([c,v,a,d])=>{content=c;verbs=v;audioMap=a;adventureData=d;for(const t of c.topics)for(const i of t.items){const old=i.id.replace(/^ar-/, '');if(old!==i.id&&p.items[old]&&!p.items[i.id]){p.items[i.id]=p.items[old];delete p.items[old];}}save();});
window.gramaticaAudio=text=>audioMap[text]?`audio/es-AR-TomasNeural/${audioMap[text]}.wav`:null;
function completeTheory(b){
 const t=content.topics.find(t=>t.id===b.topic);const part=t?.theory_parts?.find(x=>x.key===b.part);if(!part)throw Error('Unbekannter Theorieteil');
 p.theory ||= {};p.theory[t.id] ||= [];let xp=0,gold=0;const before=window.RPG.level(p.xp);
 if(!p.theory[t.id].includes(part.key)){p.theory[t.id].push(part.key);xp=8;gold=2;
 const g=window.RPG.ensure(p);g.days[day()] ||= {practice:[],new:[],review:[],theory:[]};g.days[day()].theory ||= [];g.days[day()].theory.push(t.id+':'+part.key);
 if(t.theory_parts.every(x=>p.theory[t.id].includes(x.key))){xp+=15;gold+=5;}g.gold+=gold;addXP(xp);save();}
 return {chapter_complete:xp>8,xp_gained:xp,gold_gained:gold,done:p.theory[t.id],level_up:window.RPG.level(p.xp)>before,player_level:window.RPG.level(p.xp),stats:stats()};
}
const rawApi=async(url,b)=>{await window.gramaticaReady;const u=new URL(url,location.origin);if(u.pathname.startsWith('/api/adventure'))return window.Adventure.api(p,content,adventureData,u.pathname,b||{},{date:day(),save,addXP,practiceItem,norm,strip,stats,check});switch(u.pathname){case '/api/theory':return completeTheory(b);case '/api/game':return window.RPG.snapshot(p,content,day());case '/api/game-action':{const oldGold=window.RPG.ensure(p).gold;const r=window.RPG.action(p,content,day(),b);save();return {...r,audio_claimed:b.action==='claim'&&r.gold>oldGold};}case '/api/overview':return overview();case '/api/session':return session(u.searchParams.get('topic'));case '/api/verbs':return verbs;case '/api/check':return check(b);case '/api/hint':{const i=practiceItem(all().find(x=>x.i.id===b.id).i);return {text:i.answers[0].split(' ')[0]+' …'};}case '/api/explain':return explain(b.given,b.expected);case '/api/goal':p.daily_goal=Math.max(20,Math.min(2000,b.goal));save();return stats();case '/api/sprint':{const record=b.score>p.sprint_best;p.sprint_best=Math.max(p.sprint_best,b.score);const g=window.RPG.ensure(p);g.sprintXP ||= {};const xp=Math.min(Math.max(0,60-(g.sprintXP[day()]||0)),Math.min(200,b.score)*2);g.sprintXP[day()]=(g.sprintXP[day()]||0)+xp;addXP(xp);return {best:p.sprint_best,new_record:record,xp_gained:xp,stats:stats()};}default:throw Error('Unbekannte Aktion');}};
// A single result event prevents sounds being duplicated by redraws or resume.
window.gramaticaApi=async(url,b)=>{
 await window.gramaticaReady;const previousLevel=window.RPG.level(p.xp);const out=await rawApi(url,b);const a=window.RewardAudio;if(!a)return out;const path=new URL(url,location.origin).pathname;
 try{
 if(path==='/api/check'){a.result(out.verdict!=='wrong',{...out.audio,events:out.events,hint:!!b?.hint_used});const milestone=out.events.find(x=>x.startsWith('Lernmeilenstein:'));if(milestone)a.announce(milestone);}
 if(path==='/api/theory'&&out.xp_gained){a.reward(out.chapter_complete?'unlock':'theory');if(out.level_up)a.reward('level');}
 if(path==='/api/game-action'&&out.audio_claimed)a.reward('goal');
 if(path==='/api/adventure/talent')a.reward('unlock');
 if(path==='/api/adventure/check'){const r=out.run,f=r.feedback;a.result(f.correct,{...f.audio,hard:['boss','exam','forge'].includes(r.kind),events:f.events,hint:!!r.hintText});if(r.phase==='complete'){a.advance(1);if(r.reward.first||r.reward.xp||r.reward.improved)a.reward(r.kind==='boss'?'boss':r.kind==='exam'?'legendary':'series');else a.reward('series');if(r.reward.levelUp)a.reward('level');}else a.advance(r.correct/r.total);}
 if(path==='/api/sprint'&&window.RPG.level(p.xp)>previousLevel)a.reward('level');
 if(out.stats)a.anticipateXP(out.stats.xp);
 }catch(error){console.warn('ESlearning: Audiofeedback konnte nicht abgespielt werden.',error.message);}
 return out;
};
})();
