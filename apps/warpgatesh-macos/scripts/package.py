#!/usr/bin/env python3
"""Assemble the native app using the pinned Rust tools and host Xcode."""
import argparse, json, os, pathlib, plistlib, shutil, subprocess, tempfile, tomllib
from apple_toolchain import apple_environment
root=pathlib.Path(__file__).resolve().parents[3]
native=root/'apps/warpgatesh-macos'
parser=argparse.ArgumentParser();parser.add_argument('--sign',action='store_true');options=parser.parse_args()
version=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
config=json.loads((native/'Bundle/app.json').read_text());out=native/'.build/distribution';out.mkdir(parents=True,exist_ok=True)
app=out/'WarpgateSH.app'
def run(args,**kwargs):subprocess.run(list(map(str,args)),check=True,cwd=root,**kwargs)
for target in ['aarch64-apple-darwin','x86_64-apple-darwin']:
 run(['cargo','build','--release','--locked','--target',target,'-p','warpgatesh-cli','-p','warpgatesh-agent','-p','warpgatesh-companion'])
swift=['/usr/bin/xcrun','swift','build','--package-path',str(native),'--configuration','release','--arch','arm64','--arch','x86_64']
run(swift,env=apple_environment());binary=pathlib.Path(subprocess.check_output(swift+['--show-bin-path'],env=apple_environment(),text=True,cwd=root).strip())
if app.exists():shutil.rmtree(app)
macos=app/'Contents/MacOS';resources=app/'Contents/Resources';macos.mkdir(parents=True);resources.mkdir()
for name in ['warpgatesh','warpgatesh-agent','warpgatesh-companion']:
 run(['/usr/bin/lipo','-create',root/'target/aarch64-apple-darwin/release'/name,root/'target/x86_64-apple-darwin/release'/name,'-output',macos/name])
shutil.copy2(binary/'warpgatesh-ui',macos/'warpgatesh-ui')
shutil.copytree(binary/'WarpgateSH_WarpgateSH.bundle',resources/'WarpgateSH_WarpgateSH.bundle')
shutil.copy2(native/'Bundle/icon.icns',resources/'icon.icns')
info={'CFBundleIdentifier':config['identifier'],'CFBundleName':config['name'],'CFBundleDisplayName':config['name'],'CFBundleExecutable':'warpgatesh-ui','CFBundleShortVersionString':version,'CFBundleVersion':version,'CFBundlePackageType':'APPL','CFBundleIconFile':'icon.icns','LSMinimumSystemVersion':config['minimumSystemVersion'],'NSHighResolutionCapable':True,'NSPrincipalClass':'NSApplication','LSApplicationCategoryType':'public.app-category.utilities'}
(app/'Contents/Info.plist').write_bytes(plistlib.dumps(info))
identity=os.environ.get('APPLE_SIGNING_IDENTITY') if options.sign else '-'
if not identity:raise SystemExit('APPLE_SIGNING_IDENTITY is required for a signed release')
for path in [macos/n for n in ['warpgatesh','warpgatesh-agent','warpgatesh-companion']]+[app]:
 args=['/usr/bin/codesign','--force','--sign',identity]
 if options.sign:args+=['--options','runtime','--timestamp']
 run(args+[path])
run(['/usr/bin/codesign','--verify','--deep','--strict',app])
with tempfile.TemporaryDirectory(prefix='warpgatesh-image-') as temporary:
 stage=pathlib.Path(temporary);shutil.copytree(app,stage/'WarpgateSH.app');(stage/'Applications').symlink_to('/Applications')
 run(['/usr/bin/hdiutil','create','-volname','WarpgateSH','-srcfolder',stage,'-format','UDZO','-ov',out/f'WarpgateSH_{version}_universal.dmg'])
print(app)
