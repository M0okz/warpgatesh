"""Fetch the exact official Sparkle tools, authenticated by the release asset SHA-256."""
import hashlib
import pathlib
import subprocess
import tarfile
import tempfile
import urllib.request

VERSION = '2.10.1'
SHA256 = '1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27'


def tools_directory():
    directory = pathlib.Path(__file__).resolve().parents[1] / '.build/sparkle-tools'
    marker = directory / '.verified-sha256'
    if marker.exists() and marker.read_text().strip() == SHA256:
        return directory
    directory.mkdir(parents=True, exist_ok=True)
    url = f'https://github.com/sparkle-project/Sparkle/releases/download/{VERSION}/Sparkle-{VERSION}.tar.xz'
    with tempfile.TemporaryDirectory(prefix='warpgatesh-sparkle-') as temporary:
        archive = pathlib.Path(temporary) / 'Sparkle.tar.xz'
        urllib.request.urlretrieve(url, archive)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
            raise RuntimeError('Official Sparkle tool archive checksum mismatch')
        with tarfile.open(archive) as package:
            package.extractall(directory, filter='tar')
    marker.write_text(SHA256 + '\n')
    return directory


def sign(path):
    import os
    key = os.environ['SPARKLE_ED_PRIVATE_KEY']
    tool = tools_directory() / 'bin/sign_update'
    process = subprocess.run([str(tool), '--ed-key-file', '-', '-p', str(path)],
                             input=key, capture_output=True, text=True)
    if process.returncode:
        raise RuntimeError('Sparkle signature generation failed')
    return process.stdout.strip()
