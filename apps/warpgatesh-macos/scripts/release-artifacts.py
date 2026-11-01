#!/usr/bin/env python3
"""Finalize a stapled native app into compatible update artifacts and a DMG."""
from sparkle_tools import sign
import xml.etree.ElementTree as ET
import os,pathlib,subprocess,tempfile,shutil,tomllib,json,datetime
root=pathlib.Path(__file__).resolve().parents[3]
out=root/'apps/warpgatesh-macos/.build/distribution';app=out/'WarpgateSH.app'
version=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
identity=os.environ['APPLE_SIGNING_IDENTITY']
def run(args):subprocess.run(list(map(str,args)),cwd=root,check=True)
run(['xcrun','stapler','validate',app])
archive=out/'WarpgateSH.app.tar.gz'
run(['tar','--no-xattrs','-czf',archive,'-C',out,'WarpgateSH.app'])
run(['cargo','run','--locked','--package','warpgatesh-update-sign','--',archive])
with tempfile.TemporaryDirectory(prefix='warpgatesh-final-image-') as directory:
 stage=pathlib.Path(directory);shutil.copytree(app,stage/'WarpgateSH.app',symlinks=True);(stage/'Applications').symlink_to('/Applications')
 image=out/f'WarpgateSH_{version}_universal.dmg'
 run(['hdiutil','create','-volname','WarpgateSH','-srcfolder',stage,'-format','UDZO','-ov',image])
 run(['codesign','--force','--timestamp','--sign',identity,image])
 notes=pathlib.Path(os.environ['RELEASE_NOTES_FILE']).read_text()
 url=f'https://github.com/M0okz/warpgatesh/releases/download/v{version}/WarpgateSH.app.tar.gz'
 artifact={'url':url,'signature':pathlib.Path(str(archive)+'.sig').read_text().strip()}
 manifest={'version':version,'notes':notes,'pub_date':datetime.datetime.now(datetime.timezone.utc).isoformat(),'platforms':{'darwin-aarch64':artifact,'darwin-x86_64':artifact}}
 (out/'latest.json').write_text(json.dumps(manifest,indent=2)+'\n')

# The same notarized archive serves legacy Minisign and native Sparkle clients.
namespace='http://www.andymatuschak.org/xml-namespaces/sparkle'
ET.register_namespace('sparkle',namespace)
feed=ET.Element('rss',version='2.0');channel=ET.SubElement(feed,'channel')
ET.SubElement(channel,'title').text='WarpgateSH updates'
ET.SubElement(channel,'link').text='https://warpgatesh.homeblack.fr'
ET.SubElement(channel,'description').text='Signed macOS releases'
item=ET.SubElement(channel,'item');ET.SubElement(item,'title').text=f'WarpgateSH {version}'
ET.SubElement(item,f'{{{namespace}}}version').text=version
ET.SubElement(item,f'{{{namespace}}}minimumSystemVersion').text='13.0.0'
ET.SubElement(item,'description').text=notes
ET.SubElement(item,'enclosure',{'url':url,'type':'application/octet-stream','length':str(archive.stat().st_size),f'{{{namespace}}}edSignature':sign(archive)})
appcast=out/'appcast.xml';ET.ElementTree(feed).write(appcast,encoding='utf-8',xml_declaration=True)
sign(appcast)  # Signed feed is required by the native application's Info.plist.
