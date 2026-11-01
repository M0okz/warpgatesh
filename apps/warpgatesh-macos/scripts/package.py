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
shutil.copytree(binary/'WarpgateSH_WarpgateSH.bundle',resources/'WarpgateSH_WarpgateSH.bundle',symlinks=True)
frameworks=app/'Contents/Frameworks';frameworks.mkdir()
framework=frameworks/'Sparkle.framework'
shutil.copytree(native/'.build/artifacts/sparkle/Sparkle/Sparkle.xcframework/macos-arm64_x86_64/Sparkle.framework',framework,symlinks=True)
shutil.copy2(native/'Bundle/icon.icns',resources/'icon.icns')
shutil.copy2(native/'.build/checkouts/Sparkle/LICENSE',resources/'Sparkle-LICENSE.txt')
info={'CFBundleIdentifier':config['identifier'],'CFBundleName':config['name'],'CFBundleDisplayName':config['name'],'CFBundleExecutable':'warpgatesh-ui','CFBundleShortVersionString':version,'CFBundleVersion':version,'CFBundlePackageType':'APPL','CFBundleIconFile':'icon.icns','LSMinimumSystemVersion':config['minimumSystemVersion'],'NSHighResolutionCapable':True,'NSPrincipalClass':'NSApplication','LSApplicationCategoryType':'public.app-category.utilities'}
info.update({'SUFeedURL':'https://github.com/M0okz/warpgatesh/releases/latest/download/appcast.xml','SUPublicEDKey':(native/'Bundle/sparkle.pub').read_text().strip(),'SUEnableAutomaticChecks':False,'SUAutomaticallyUpdate':False,'SUEnableSystemProfiling':False,'SUVerifyUpdateBeforeExtraction':True,'SURequireSignedFeed':True,'SUShowReleaseNotes':False})
(app/'Contents/Info.plist').write_bytes(plistlib.dumps(info))
identity=os.environ.get('APPLE_SIGNING_IDENTITY') if options.sign else '-'
if not identity:raise SystemExit('APPLE_SIGNING_IDENTITY is required for a signed release')
# Sign nested Sparkle services inside-out; main executables are signed with their containers.
vendor=framework/'Versions/B'
nested=[vendor/'Autoupdate', *sorted(vendor.glob('XPCServices/*.xpc')), vendor/'Updater.app', framework]
for path in nested+[macos/n for n in ['warpgatesh','warpgatesh-agent','warpgatesh-companion']]+[app]:
 if not path.exists():raise SystemExit(f'Missing required code: {path}')
 args=['/usr/bin/codesign','--force','--sign',identity]
 if options.sign:args+=['--options','runtime','--timestamp']
 run(args+[path])
run(['/usr/bin/codesign','--verify','--deep','--strict',app])
with tempfile.TemporaryDirectory(prefix='warpgatesh-image-') as temporary:
 stage=pathlib.Path(temporary);shutil.copytree(app,stage/'WarpgateSH.app',symlinks=True);(stage/'Applications').symlink_to('/Applications')
 run(['/usr/bin/hdiutil','create','-volname','WarpgateSH','-srcfolder',stage,'-format','UDZO','-ov',out/f'WarpgateSH_{version}_universal.dmg'])
print(app)

run([__import__('sys').executable,native/'scripts/verify-bundle.py',app])
run(['/usr/bin/hdiutil','verify',out/f'WarpgateSH_{version}_universal.dmg'])
