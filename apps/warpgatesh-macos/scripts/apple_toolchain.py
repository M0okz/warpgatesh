"""Keep Swift/Xcode builds independent of the enclosing Rust/Nix toolchain."""
import os
import pathlib
import subprocess


def apple_environment():
    environment = os.environ.copy()
    tool_variables = {
        'LD', 'LDPLUSPLUS', 'CC', 'CXX', 'AR', 'AS', 'NM', 'RANLIB', 'STRIP',
        'CFLAGS', 'CXXFLAGS', 'CPPFLAGS', 'LDFLAGS', 'SDKROOT',
    }
    for key in list(environment):
        if key in tool_variables or key.startswith('NIX_'):
            environment.pop(key)
    environment['PATH'] = '/usr/bin:/bin:/usr/sbin:/sbin'

    def find(tool):
        return subprocess.check_output(
            ['/usr/bin/xcrun', '--find', tool], env=environment, text=True,
        ).strip()

    swift = find('swift')
    clang = find('clang')
    clangxx = find('clang++')
    environment['PATH'] = str(pathlib.Path(swift).parent) + ':' + environment['PATH']
    environment['CC'] = clang
    environment['CXX'] = clangxx
    # Xcode's LD setting is a compiler driver; a raw `ld` cannot accept -Xlinker.
    environment['LD'] = clang
    environment['LDPLUSPLUS'] = clangxx
    environment['SDKROOT'] = subprocess.check_output(
        ['/usr/bin/xcrun', '--sdk', 'macosx', '--show-sdk-path'],
        env=environment, text=True,
    ).strip()
    environment['MACOSX_DEPLOYMENT_TARGET'] = '13.0'
    return environment
