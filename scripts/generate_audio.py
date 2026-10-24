"""Generate complete Argentine male recordings. Never silently substitute another voice."""
import asyncio,hashlib,json,re,subprocess,sys,shutil
from pathlib import Path
import edge_tts
ROOT=Path(sys.argv[1] if len(sys.argv)>1 else 'web')
VOICE='es-AR-TomasNeural'
OUT=ROOT/'audio'/VOICE
OUT.mkdir(parents=True,exist_ok=True)
content=json.loads((ROOT/'content.json').read_text())
texts={'¡Hola! Vamos a practicar español.'}
for t in content['topics']:
 for i in t['items']:
  text=i['answers'][0]
  if i['type'] in ('cloze','choice') and '___' in i['prompt']:
   text=re.sub(r' \([^)]*\)','',i['prompt'].replace('___',text).replace(' → ', ', '),count=1)
  texts.add(text)
mapping={text:hashlib.sha256((VOICE+':'+text).encode()).hexdigest()[:24] for text in sorted(texts)}
(ROOT/'audio-map.json').write_text(json.dumps(mapping,ensure_ascii=False,indent=2))
sem=asyncio.Semaphore(3)
async def generate(text,hash):
 target=OUT/(hash+'.wav')
 if target.exists() and target.stat().st_size>1000:return
 async with sem:
  for attempt in range(4):
   temp=OUT/(hash+'.mp3');padded=OUT/(hash+'.tmp.wav')
   try:
    await asyncio.wait_for(edge_tts.Communicate(text,VOICE,rate='-5%').save(str(temp)),timeout=45)
    # Decode without another lossy encoding and add a quiet tail for complete sentence endings.
    subprocess.run([shutil.which('ffmpeg') or __import__('imageio_ffmpeg').get_ffmpeg_exe(),'-hide_banner','-loglevel','error','-y','-i',str(temp),'-af','apad=pad_dur=0.4','-c:a','pcm_s16le',str(padded)],check=True)
    padded.replace(target);temp.unlink(missing_ok=True)
    print('Generated',hash,flush=True);return
   except Exception:
    temp.unlink(missing_ok=True);padded.unlink(missing_ok=True)
    if attempt==3:raise
    await asyncio.sleep(2**(attempt+1))
async def main():
 await asyncio.gather(*(generate(t,h) for t,h in mapping.items()))
 print(f'Complete: {len(mapping)} male Argentine recordings.')
if __name__=='__main__':asyncio.run(main())
