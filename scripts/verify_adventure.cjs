'use strict';
const fs=require('fs'),vm=require('vm'),assert=require('assert'),path=require('path');
const root=path.resolve(process.argv[2]||'web');let saved;const window={};const ctx={window,localStorage:{getItem:()=>null,setItem:(key,text)=>{saved=JSON.parse(text);}},fetch:async url=>({ok:true,json:async()=>JSON.parse(fs.readFileSync(path.join(root,url.split('?')[0]),'utf8'))}),URL,location:{origin:'https://einfachsony.github.io'},alert:()=>{}};
for(const name of ['rpg.js','adventure.js','local-api.js'])vm.runInNewContext(fs.readFileSync(path.join(root,name),'utf8'),ctx);
const api=(name,body)=>window.gramaticaApi('/api/adventure'+name,body);
const next=run=>api('/next',{run:run.id});
async function solve(id,topic){let run=(await api('/start',{id,topic})).run;while(run.phase==='ask'){const q=saved.adventure.active.questions[run.index];assert(!('answers' in run.question));assert(window.gramaticaAudio(q.audioText),'Missing reply audio');if(q.spoken)assert(window.gramaticaAudio(q.spoken),'Missing NPC audio');if(q.mode==='mission')assert(!q.answers[0].includes('→'));if(q.mode==='gaps')assert((q.gapped.match(/____/g)||[]).length>=2);run=(await api('/check',{run:run.id,answer:q.answers[0]})).run;if(run.phase==='feedback')run=(await next(run)).run;}assert.equal(run.phase,'complete');return run;}
(async()=>{
 await window.gramaticaReady;
 await assert.rejects(()=>api('/start',{id:'buenos-aires-boss'}),/zuerst/);
 await assert.rejects(()=>api('/talent',{id:'grammar-guard'}),/Talentpunkt/);
 let r=(await api('/start',{id:'buenos-aires-talk'})).run;
 r=(await api('/check',{run:r.id,answer:'Yo soy Sony.'})).run;
 await assert.rejects(()=>api('/check',{run:r.id,answer:'Soy Sony.'}),/bereits/);
 r=(await next(r)).run;r=(await api('/check',{run:r.id,answer:'Quiero un café, por favor.'})).run;assert(r.feedback.expected.includes('café'));assert(window.gramaticaAudio(r.feedback.expected));r=(await next(r)).run;
 assert(r.question.prompt.includes('Milch'),'Coffee order must branch');
 assert.equal((await api('/resume',{})).run.id,r.id);
 r=(await api('/check',{run:r.id,answer:'Un café con leche, por favor.'})).run;
 assert.equal(r.reward.xp,65);assert.equal(r.reward.points,1);assert.equal(r.stars,3);
 await assert.rejects(()=>api('/check',{run:r.id,answer:'Quiero un café con leche.'}),/bereits/);
 const replay=await solve('buenos-aires-talk');assert.equal(replay.reward.xp,0);assert.equal(replay.reward.gold,0);assert.equal(replay.reward.points,0);
 await api('/talent',{id:'grammar-guard'});assert.equal((await api('')).points,0);
 r=(await api('/start',{id:'buenos-aires-boss'})).run;assert.equal(r.maxHearts,4);assert.equal(new Set(saved.adventure.active.questions.map(q=>q.audioText)).size,3,'Boss phases must use distinct answers');
 for(let n=0;n<4;n++){r=(await api('/check',{run:r.id,answer:'wrong answer'})).run;assert.equal(r.index,0);if(n<3)r=(await next(r)).run;}
 assert.equal(r.phase,'failed');assert(!saved.adventure.clears['buenos-aires-boss']);
 r=await solve('buenos-aires-boss');assert.equal(r.reward.xp,120);assert.equal(r.reward.points,1);assert((await api('')).regions[1].open);
 await api('/equip',{id:'buenos-aires-boss'});assert.equal((await api('')).equipped,'buenos-aires-boss');
 await assert.rejects(()=>api('/equip',{id:'fake'}));
 const data=JSON.parse(fs.readFileSync(path.join(root,'adventure.json')));
 for(const region of data.regions.slice(1)){await solve(region.id+'-talk');await solve(region.id+'-boss');}
 assert.equal((await api('')).bosses,6);assert.equal((await api('')).inventory.length,12);
 await api('/talent',{id:'hearing-slow'});await api('/talent',{id:'hearing-guard'});
 r=(await api('/start',{id:'radio'})).run;assert(r.slow);assert.equal(r.maxHearts,4);
 r=await solve('radio');assert.equal(r.reward.xp,45);r=await solve('radio');assert.equal(r.reward.xp,0);
 await solve('forge');await solve('defend');
 await assert.rejects(()=>api('/start',{id:'exam',topic:'art'}),/Theorieteile/);
 const content=JSON.parse(fs.readFileSync(path.join(root,'content.json')));
 for(const part of content.topics[0].theory_parts)await window.gramaticaApi('/api/theory',{topic:'art',part:part.key});
 await solve('exam','art');
 await api('/talent',{id:'talk-clue'});
 r=(await api('/start',{id:'buenos-aires-talk'})).run;r=(await api('/hint',{run:r.id})).run;assert.equal(r.hints,0);r=(await api('/hint',{run:r.id})).run;assert.equal(r.hints,0);
 r=(await api('/check',{run:r.id,answer:'Soy Soner.'})).run;r=(await next(r)).run;r=(await api('/hint',{run:r.id})).run;assert.equal(r.hints,1);
 const snapshot=JSON.stringify(saved);window.gramaticaImport(snapshot);assert((await api('')).talents.find(t=>t.id==='hearing-slow').owned);assert.equal((await api('/resume',{})).run.id,r.id);
 console.log('PASS: all 12 route missions, dialogue branching, 4 training modes, locked progression, failure/retry, resume/import, talents, equipment, audio coverage and duplicate-reward protection.');
})().catch(err=>{console.error(err);process.exit(1);});
