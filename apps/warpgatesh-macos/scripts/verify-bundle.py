#!/usr/bin/env python3
"""Check the final native bundle, including both executable slices."""
import pathlib,plistlib,subprocess,sys,tomllib
root=pathlib.Path(__file__).resolve().parents[3]
app=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root/'apps/warpgatesh-macos/.build/distribution/WarpgateSH.app'
version=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
info=plistlib.loads((app/'Contents/Info.plist').read_bytes())
assert info['CFBundleIdentifier']=='dev.warpgatesh.companion' and info['CFBundleExecutable']=='warpgatesh-ui'
assert info['CFBundleShortVersionString']==version and info['LSMinimumSystemVersion']=='13.0'
for name in ['warpgatesh-ui','warpgatesh-companion','warpgatesh','warpgatesh-agent']:
 binary=app/'Contents/MacOS'/name
 assert set(subprocess.check_output(['/usr/bin/lipo','-archs',str(binary)],text=True).split())=={'arm64','x86_64'}
 for arch in ['arm64','x86_64']:
  commands=subprocess.check_output(['/usr/bin/otool','-arch',arch,'-l',str(binary)],text=True)
  assert '/nix/store/' not in commands
  assert 'minos 13.0' in commands
 assert '/nix/store/' not in subprocess.check_output(['/usr/bin/otool','-L',str(binary)],text=True)
assert subprocess.check_output([str(app/'Contents/MacOS/warpgatesh'),'--version'],text=True).strip()==f'warpgatesh {version}'
assert (app/'Contents/Resources/WarpgateSH_WarpgateSH.bundle').is_dir()
subprocess.run(['/usr/bin/codesign','--verify','--deep','--strict',str(app)],check=True)
signature=subprocess.run(['/usr/bin/codesign','-dvv',str(app/'Contents/MacOS/warpgatesh-ui')],capture_output=True,text=True,check=True).stderr
assert 'Identifier=dev.warpgatesh.companion' in signature
print(f'Verified native universal WarpgateSH {version}')
