#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn commands_use_one_short_name_even_when_ssh_short_alias_is_blocked() {
    let directory = tempfile::tempdir().unwrap();
    let support = if cfg!(target_os = "macos") {
        directory
            .path()
            .join("Library/Application Support/WarpgateSH")
    } else {
        directory.path().join(".local/share/warpgatesh")
    };
    fs::create_dir_all(&support).unwrap();
    fs::write(support.join("profiles.json"), r#"{
        "schema_version":1,"default_profile":"hblk","profiles":[
            {"name":"hblk","base_url":"https://example.invalid","username":"fixture","ssh_host":"example.invalid","ssh_port":2222},
            {"name":"other","base_url":"https://other.invalid","username":"fixture","ssh_host":"other.invalid","ssh_port":2222}
        ]
    }"#).unwrap();
    fs::write(support.join("snapshot.json"), r#"{
        "schema_version":1,"synchronized_at_epoch_seconds":1,"targets":[
            {"profile":"hblk","target_id":"1","name":"trust-admin-01","short_alias":null,"qualified_alias":"trust-admin-01.hblk"},
            {"profile":"other","target_id":"2","name":"trust-admin-01","short_alias":null,"qualified_alias":"trust-admin-01.other"}
        ]
    }"#).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_warpgatesh"))
        .args(["shell", "commands"])
        .env("HOME", directory.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "trust-admin-01\ttrust-admin-01.hblk\ntrust-admin-01.other\ttrust-admin-01.other\n"
    );
}

#[test]
fn zsh_refresh_preserves_personal_commands_and_forwards_arguments() {
    let directory = tempfile::tempdir().unwrap();
    let executable = directory.path().join("warpgatesh");
    let shell = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join("sh"))
        .find(|path| path.is_file())
        .expect("a POSIX shell must be available on PATH");
    fs::write(
        &executable,
        format!("#!{}\nif [ \"$1\" = shell ]; then cat \"$TARGETS\"; else printf '%s\\n' \"$@\" > \"$CALL\"; fi\n", shell.display()),
    ).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let targets = directory.path().join("targets");
    fs::write(
        &targets,
        "trust-admin-01\ttrust-admin-01.hblk\nls\tls.hblk\ncd\tcd.hblk\npersonal\tpersonal.hblk\ncustom\tcustom.hblk\nold-target\told-target.hblk\ninvalid;name\tvalid.hblk\ninvalid-destination\tbad;destination\n",
    )
    .unwrap();
    let mut paths = vec![directory.path().to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let result = Command::new("zsh")
        .args(["-f", "-c", r#"
            alias personal='echo mine'
            custom() { echo original; }
            source "$INTEGRATION"
            (( $+functions[trust-admin-01] )) || exit 1
            (( $+functions[trust-admin-01.hblk] )) && exit 2
            (( $+functions[ls.hblk] && $+functions[personal.hblk] )) || exit 11
            (( $+functions[invalid-destination] )) && exit 12
            (( $+functions[ls] || $+functions[cd] || $+functions[personal] )) && exit 3
            [[ $(custom) == original ]] || exit 4
            trust-admin-01 -L '8080:localhost:80' 'printf hello'
            [[ $(cat "$CALL") == $'trust-admin-01.hblk\n--\n-L\n8080:localhost:80\nprintf hello' ]] || exit 5
            old-target() { echo replacement; }
            print -r -- $'new-target\tnew-target.hblk' > "$TARGETS"
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
        .env("PATH", std::env::join_paths(paths).unwrap())
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
