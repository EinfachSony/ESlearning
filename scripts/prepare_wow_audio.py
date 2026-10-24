"""Fetch the four public Wowhead download URLs; verify immutable bytes before publishing.
Original files are preserved. PCM WAV alternatives support browsers without Ogg/Vorbis.
No runtime dependency on third-party CORS or CDN availability after deployment.
"""
import hashlib,json,pathlib,sys,urllib.request,concurrent.futures,subprocess,shutil
root=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else 'web')
manifest=json.loads((root/'wow-sounds.json').read_text())
out=root/'audio'/'wow';out.mkdir(parents=True,exist_ok=True)
def prepare(pair):
    ident,entry=pair
    data=urllib.request.urlopen(entry['url'],timeout=45).read()
    if not data.startswith(b'OggS'):raise ValueError('Not an Ogg file: '+ident)
    digest=hashlib.sha256(data).hexdigest()
    if entry.get('sha256') and digest!=entry['sha256']:raise ValueError('Source changed: '+ident)
    (out/entry['file']).write_bytes(data)
    return ident,len(data),digest
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
    for ident,size,digest in pool.map(prepare,manifest['sounds'].items()):
        print(ident,size,digest)
ffmpeg=shutil.which('ffmpeg')
if not ffmpeg:
    import imageio_ffmpeg
    ffmpeg=imageio_ffmpeg.get_ffmpeg_exe()
for ident,e in manifest['sounds'].items():
    subprocess.run([ffmpeg,'-v','error','-y','-i',str(out/e['file']),'-c:a','pcm_s16le',str(out/(ident+'.wav'))],check=True)
print('Prepared original WoW recordings and lossless decoded alternatives.')
