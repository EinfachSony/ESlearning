/* Prepare score PCM off the UI thread; transfer sample arrays without copies. */
importScripts('sound-score.js?v=20261024-sound1');
function send(id,pcm){postMessage({id,pcm},[pcm.left.buffer,pcm.right.buffer]);}
for(const kind of ['ui','error','correct'])for(let v=0;v<3;v++)send(kind+':1:'+v,ESScore.render(ESScore.score(kind,1,v)));
for(let n=1;n<=20;n++)send('combo:'+n+':0',ESScore.render(ESScore.score('combo',n)));
for(let k=0;k<5;k++)send('stem:'+k,ESScore.render(ESScore.stem(k)));
for(const kind of ['difficult','theory','mastery','goal','series','unlock','level','boss','legendary','anticipate'])for(let v=0;v<3;v++)send(kind+':1:'+v,ESScore.render(ESScore.score(kind,1,v)));
