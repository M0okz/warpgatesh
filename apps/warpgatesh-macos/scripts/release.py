#!/usr/bin/env python3
"""Build, notarize and validate native distribution outputs without publishing."""
import os,pathlib,subprocess,sys,tempfile,time,tomllib,hashlib
root=pathlib.Path(__file__).resolve().parents[3];scripts=root/'apps/warpgatesh-macos/scripts';out=root/'apps/warpgatesh-macos/.build/distribution'
version=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
for key in ['APPLE_SIGNING_IDENTITY','APPLE_ID','APPLE_PASSWORD','APPLE_TEAM_ID','TAURI_SIGNING_PRIVATE_KEY','TAURI_SIGNING_PRIVATE_KEY_PASSWORD','RELEASE_NOTES_FILE','SPARKLE_ED_PRIVATE_KEY']:
 if key not in os.environ:raise SystemExit(f'Missing required release input: {key}')
def run(args, retry=False):
 for attempt in range(6 if retry else 1):
  code=subprocess.run(list(map(str,args)),cwd=root,check=False).returncode
  if code==0:return
  if not retry or code!=68 or attempt==5:raise SystemExit('Distribution command failed: '+str(args[0]))
  time.sleep(5)
def notarize(path):
 run(['xcrun','notarytool','submit',path,'--apple-id',os.environ['APPLE_ID'],'--password',os.environ['APPLE_PASSWORD'],'--team-id',os.environ['APPLE_TEAM_ID'],'--wait'])
run([sys.executable,scripts/'package.py','--sign'])
app=out/'WarpgateSH.app'
with tempfile.TemporaryDirectory(prefix='warpgatesh-notary-') as directory:
 archive=pathlib.Path(directory)/'WarpgateSH.zip';run(['ditto','-c','-k','--keepParent',app,archive]);notarize(archive)
run(['xcrun','stapler','staple',app],retry=True);run(['xcrun','stapler','validate',app],retry=True)
run([sys.executable,scripts/'release-artifacts.py'])
image=out/f'WarpgateSH_{version}_universal.dmg';notarize(image)
run(['xcrun','stapler','staple',image],retry=True);run(['xcrun','stapler','validate',image],retry=True)
run(['hdiutil','verify',image]);run([sys.executable,scripts/'verify-bundle.py',app]);run(['spctl','--assess','--type','execute',app])
with tempfile.TemporaryDirectory(prefix='warpgatesh-updater-check-') as directory:
 run(['tar','-xzf',out/'WarpgateSH.app.tar.gz','-C',directory]);extracted=pathlib.Path(directory)/'WarpgateSH.app'
 run([sys.executable,scripts/'verify-bundle.py',extracted]);run(['xcrun','stapler','validate',extracted],retry=True)
pathlib.Path(str(image)+'.sha256').write_text(hashlib.sha256(image.read_bytes()).hexdigest()+'  '+image.name+'\n')
print('Signed, notarized distribution is ready for draft publication.')
