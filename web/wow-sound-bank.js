/* Original World of Warcraft clips, credited in wow-sounds.json. No pitch shifting. */
(()=>{'use strict';
const names=['coin','quest','level','achievement'],buffers=new Map();let preparation=null,status='loading';
const encoded=Promise.all(names.map(async name=>{const r=await fetch('audio/wow/'+name+'.wav?v=20261024-wow1');if(!r.ok)throw Error('Aufnahme fehlt: '+name);return [name,await r.arrayBuffer()];})).catch(error=>{status='failed';return null;});
function update(){const el=document.getElementById('wow-audio-status');if(el)el.textContent=status==='ready'?'Vier Original-WoW-Aufnahmen bereit.':status==='failed'?'WoW-Aufnahmen konnten nicht geladen werden. Bitte lade die Seite erneut.':'WoW-Aufnahmen werden vorbereitet …';}
function prepare(ctx){if(preparation)return preparation;preparation=(async()=>{const files=await encoded;if(!files){update();return false;}try{const decoded=await Promise.all(files.map(async([name,bytes])=>[name,await ctx.decodeAudioData(bytes)]));for(const [name,b] of decoded)buffers.set(name,b);status='ready';update();return true;}catch{status='failed';update();return false;}})();return preparation;}
function clip(kind){return ['correct','difficult','combo','theory','mastery','ui'].includes(kind)?'coin':['goal','series'].includes(kind)?'quest':kind==='level'?'level':['unlock','boss','legendary'].includes(kind)?'achievement':null;}
function gain(kind,n=1){return kind==='ui'?.16:['correct','difficult','combo','theory','mastery'].includes(kind)?(n%5===0?.55:.42):.72;}
window.WoWSoundBank={prepare,get:(kind)=>buffers.get(clip(kind))||null,gain,update,get status(){return status;}};
})();
