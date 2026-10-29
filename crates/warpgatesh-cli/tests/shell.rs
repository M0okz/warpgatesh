#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn zsh_refresh_preserves_personal_commands_and_forwards_arguments() {
    let directory = tempfile::tempdir().unwrap();
    let executable = directory.path().join("warpgatesh");
    fs::write(
        &executable,
        "#!/bin/sh\nif [ \"$1\" = shell ]; then cat \"$TARGETS\"; else printf '%s\\n' \"$@\" > \"$CALL\"; fi\n",
    ).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let targets = directory.path().join("targets");
    fs::write(
        &targets,
        "trust-admin-01\ntrust-admin-01.homeblack\nls\ncd\npersonal\ncustom\nold-target\n",
    )
    .unwrap();
    let result = Command::new("zsh")
        .args(["-f", "-c", r#"
            alias personal='echo mine'
            custom() { echo original; }
            source "$INTEGRATION"
            (( $+functions[trust-admin-01] )) || exit 1
            (( $+functions[trust-admin-01.homeblack] )) || exit 2
            (( $+functions[ls] || $+functions[cd] || $+functions[personal] )) && exit 3
            [[ $(custom) == original ]] || exit 4
            trust-admin-01 -L '8080:localhost:80' 'printf hello'
            [[ $(cat "$CALL") == $'trust-admin-01\n--\n-L\n8080:localhost:80\nprintf hello' ]] || exit 5
            old-target() { echo replacement; }
            print -r -- new-target > "$TARGETS"
            _warpgatesh_refresh
            (( $+functions[trust-admin-01] )) && exit 6
            (( $+functions[new-target] )) || exit 7
            [[ $(old-target) == replacement ]] || exit 8
            source "$INTEGRATION"
            (( ${#precmd_functions[(r)_warpgatesh_refresh]} > 0 )) || exit 9
            rm "$TARGETS"
            _warpgatesh_refresh 2>/dev/null
            (( $+functions[new-target] )) || exit 10
            exit 0
        "#])
        .env("PATH", format!("{}:/usr/bin:/bin", directory.path().display()))
        .env("TARGETS", &targets)
        .env("CALL", directory.path().join("call"))
        .env("INTEGRATION", concat!(env!("CARGO_MANIFEST_DIR"), "/src/shell.zsh"))
        .output().unwrap();
    assert!(
        result.status.success(),
        "status={}\n{}",
        result.status,
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn cli_install_uninstall_honors_zdotdir_and_preserves_permissions() {
    let directory = tempfile::tempdir().unwrap();
    let startup = directory.path().join(".zshrc");
    let original = "# User configuration\nexport EDITOR=vim\n";
    fs::write(&startup, original).unwrap();
    fs::set_permissions(&startup, fs::Permissions::from_mode(0o640)).unwrap();
    for operation in ["install", "install", "uninstall"] {
        let result = Command::new(env!("CARGO_BIN_EXE_warpgatesh"))
            .args(["shell", operation, "zsh"])
            .env("ZDOTDIR", directory.path())
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            fs::metadata(&startup).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
    assert_eq!(fs::read_to_string(&startup).unwrap(), original);
    fs::remove_file(&startup).unwrap();
    std::os::unix::fs::symlink(directory.path().join("personal"), &startup).unwrap();
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_warpgatesh"))
            .args(["shell", "install", "zsh"])
            .env("ZDOTDIR", directory.path())
            .output()
            .unwrap()
            .status
            .success()
    );
}
