/* Original ESlearning score. D major / B minor, 80 BPM. No sampled game assets. */
(function(root){
'use strict';
const rate=24000,beat=.75;
const midi=n=>440*2**((n-69)/12);
function score(kind,step=1,variant=0){
 let notes=[],duration=.45;const add=(at,pitch,len=.5,amp=.22,voice='bell',pan=0)=>notes.push({at,pitch,len,amp,voice,pan});
 const chord=(at,pitches,len,amp=.16,voice='pad')=>pitches.forEach((n,i)=>add(at,n,len,amp,voice,(i-1)*.24));
 const motif=[62,66,69,73,74],offset=variant%3===1?.012:variant%3===2?-.012:0;
 if(kind==='ui'){duration=.17;add(0,69,.14,.12,'pluck');}
 else if(kind==='error'){duration=.4;add(0,59,.32,.14,'wood');add(.075,62,.28,.09,'wood');}
 else if(kind==='anticipate'){duration=.7;add(0,69,.4,.12);add(.17,73,.45,.1);}
 else if(kind==='correct'||kind==='difficult'||kind==='combo'){
   let count=kind==='combo'?Math.min(5,Math.max(1,step%5||5)):kind==='difficult'?3:1;
   duration=kind==='combo'&&step>=10?1.6:kind==='combo'&&step%5===0?.95:Math.min(.5,.22+count*.07);
   for(let k=0;k<count;k++)add(k*.065,motif[(k+(step>=6?1:0))%5]+(step>=15?12:0),duration-k*.065,.19,'bell',k*.1-.2);
   if(kind==='combo'&&step%5===0)chord(.18,[50,57,62,66],duration-.2,.1);
   if(step>=10&&kind==='combo'&&step%5===0){[62,66,69,74].forEach((n,k)=>add(.25+k*.12,n,.75,.17,'harp'));}
 }
 else if(['theory','mastery','goal','series','unlock'].includes(kind)){
   duration=kind==='unlock'?2.4:1.45;
   [62,66,69,74].forEach((n,k)=>add(k*.13,n,.8,.2,'harp',k*.15-.2));chord(.22,[50,57,66],duration-.3,.09);
   if(kind==='mastery')add(.65,81,.65,.12);
 }
 else {
   duration=kind==='legendary'?6.4:kind==='boss'?5.4:3.4;
   const line=kind==='legendary'?[62,66,69,73,74,78,81,86]:[62,66,69,74,73,74];
   chord(0,[38,50,57,62],1.5,.1);chord(1.1,[43,55,59,66],1.6,.1);chord(2.35,[38,50,57,66,74],duration-2.4,.1);
   line.forEach((n,k)=>add(.14+k*.23,n,1.15,.22,'horn',(k%3-1)*.2));
   [0,.75,1.5,2.4].forEach(t=>add(t,38,.5,.2,'drum'));
   if(duration>4)[74,78,81,86].forEach((n,k)=>add(3+k*.19,n,duration-3-k*.19,.18,'bell',(k-1.5)*.25));
 }
 return {duration,peak:(kind==='ui'?.1:kind==='error'?.16:kind==='anticipate'?.13:duration>2?.5:duration>.5?.34:.28)*[1,.97,1.025][variant%3],notes:notes.map(n=>({...n,at:Math.max(0,n.at+offset),amp:n.amp*[1,.97,1.025][variant%3]}))};
}
function stem(layer){
 const notes=[],duration=24;const add=(at,pitch,len,amp,voice,pan=0)=>notes.push({at,pitch,len,amp,voice,pan});
 const chords=[[50,57,62,66],[47,54,59,62],[43,55,59,66],[45,57,61,64]];
 for(let bar=0;bar<8;bar++){const c=chords[Math.floor(bar/2)];const at=bar*3;
  if(layer===0){add(at,c[0]-12,2.95,.13,'pad',-.12);[0,1].forEach(k=>add(at+k*1.5,[74,73,69,66,69,74,73,69][bar]+(k? -4:0),1.35,.075,'flute',.15));}
  if(layer===1)c.slice(1).forEach((n,k)=>add(at,n,2.95,.075,'pad',(k-1)*.3));
  if(layer===2)[0,1.5].forEach(t=>add(at+t,38,.35,.08,'drum'));
  if(layer===3)for(let k=0;k<4;k++)add(at+k*beat,c[1+k%3]+12,.65,.075,'harp',(k-1.5)*.18);
  if(layer===4){add(at,c[0],2.95,.065,'horn',-.15);add(at+1.5,c[2]+12,1.4,.07,'bell',.2);}
 }
 return {duration,notes};
}
function render(s,sampleRate=rate){
 const size=Math.ceil(s.duration*sampleRate),left=new Float32Array(size),right=new Float32Array(size);
 for(const n of s.notes){const start=Math.floor(n.at*sampleRate),length=Math.min(Math.ceil(n.len*sampleRate),size-start),f=midi(n.pitch),pan=n.pan||0;
  for(let j=0;j<length;j++){
   const t=j/sampleRate,x=t/n.len,attack=Math.min(1,t/(n.voice==='pad'?.12:n.voice==='horn'?.045:.007));
   const release=Math.min(1,(n.len-t)/.08);let v,env;
   if(n.voice==='bell'){v=Math.sin(2*Math.PI*f*t)*.74+Math.sin(2*Math.PI*f*2*t)*.19*Math.exp(-t*6)+Math.sin(2*Math.PI*f*3*t)*.07*Math.exp(-t*12);env=Math.exp(-x*5);}
   else if(n.voice==='harp'||n.voice==='pluck'){v=Math.sin(2*Math.PI*f*t)+.2*Math.sin(2*Math.PI*f*2*t)+.07*Math.sin(2*Math.PI*f*3*t);env=Math.exp(-x*5);}
   else if(n.voice==='drum'){v=Math.sin(2*Math.PI*(55*t+8*(1-Math.exp(-t*22))));env=Math.exp(-x*8);}
   else if(n.voice==='horn'){v=.72*Math.sin(2*Math.PI*f*t)+.18*Math.sin(2*Math.PI*f*2*t)+.08*Math.sin(2*Math.PI*f*3*t);env=Math.sin(Math.PI*x)**.7;}
   else if(n.voice==='wood'){v=Math.sin(2*Math.PI*f*t)+.12*Math.sin(2*Math.PI*f*3*t);env=Math.exp(-x*6);}
   else {v=Math.sin(2*Math.PI*f*t+.004*Math.sin(2*Math.PI*4*t))+.12*Math.sin(2*Math.PI*f*2*t);env=Math.sin(Math.PI*x)**.8;}
   const value=v*env*attack*release*n.amp*.5;
   left[start+j]+=value*Math.sqrt((1-pan)/2);right[start+j]+=value*Math.sqrt((1+pan)/2);
  }
 }
 // Short, quiet stereo early reflections; no cross-loop tails or hard edges.
 for(let ch of [left,right]){const delay=Math.floor(sampleRate*(ch===left?.071:.097));for(let i=size-1;i>=delay;i--)ch[i]+=ch[i-delay]*.12;for(let i=0;i<Math.min(240,size);i++){const g=i/240;ch[i]*=g;ch[size-1-i]*=g;}}
 if(s.peak){let peak=0;for(let i=0;i<size;i++)peak=Math.max(peak,Math.abs(left[i]),Math.abs(right[i]));const gain=Math.min(5,s.peak/Math.max(.001,peak));for(let ch of [left,right])for(let i=0;i<size;i++)ch[i]*=gain;}
 return {sampleRate,left,right,duration:s.duration};
}
const api={score,stem,render,rate,beat};if(typeof module!=='undefined')module.exports=api;else root.ESScore=api;
})(typeof window==='undefined'?globalThis:window);
