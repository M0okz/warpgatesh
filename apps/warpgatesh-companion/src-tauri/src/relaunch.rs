use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tauri::Manager;

// LaunchServices must see the old process gone before opening the replaced bundle.
// Pass paths and arguments as positional parameters, never as shell source.
const RELAUNCH_SCRIPT: &str = r#"
parent_pid="$1"
application="$2"
shift 2
attempts=0
while /bin/kill -0 "$parent_pid" 2>/dev/null; do
  if [ "$attempts" -ge 600 ]; then exit 1; fi
  attempts=$((attempts + 1))
  /bin/sleep 0.1
done
exec /usr/bin/open -n "$application" --args "$@"
"#;

pub(crate) struct RelaunchPlan {
    application: PathBuf,
    arguments: Vec<OsString>,
}

impl RelaunchPlan {
    /// Resolve the original bundle before the updater moves or removes its executable.
    pub(crate) fn prepare(app: &tauri::AppHandle) -> Result<Self, String> {
        let environment = app.env();
        let executable = tauri::process::current_binary(&environment)
            .map_err(|error| format!("Impossible de préparer le redémarrage : {error}"))?;
        Ok(Self {
            application: application_bundle(&executable)?,
            arguments: environment.args_os.into_iter().skip(1).collect(),
        })
    }

    pub(crate) fn restart(self, app: &tauri::AppHandle) -> Result<(), String> {
        if !self.application.join("Contents/Info.plist").is_file() {
            return Err(
                "Le paquet mis à jour est introuvable. Rouvrez WarpgateSH depuis Applications."
                    .to_owned(),
            );
        }
        relaunch_command(&self.application, std::process::id(), &self.arguments)
            .spawn()
            .map_err(|error| format!("Impossible de programmer le redémarrage : {error}"))?;
        app.exit(0);
        Ok(())
    }
}

fn application_bundle(executable: &Path) -> Result<PathBuf, String> {
    let macos = executable.parent();
    let contents = macos.and_then(Path::parent);
    let bundle = contents.and_then(Path::parent);
    match (macos, contents, bundle) {
        (Some(macos), Some(contents), Some(bundle))
            if macos.file_name().is_some_and(|name| name == "MacOS")
                && contents.file_name().is_some_and(|name| name == "Contents")
                && bundle
                    .extension()
                    .is_some_and(|extension| extension == "app") =>
        {
            Ok(bundle.to_owned())
        }
        _ => Err("Le redémarrage nécessite une application macOS installée.".to_owned()),
    }
}

fn relaunch_command(application: &Path, parent_pid: u32, arguments: &[OsString]) -> Command {
    use std::os::unix::process::CommandExt;

    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", RELAUNCH_SCRIPT, "warpgatesh-relaunch"])
        .arg(parent_pid.to_string())
        .arg(application)
        .args(arguments)
        .current_dir("/")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_bundle_without_requiring_the_old_executable_to_exist() {
        assert_eq!(
            application_bundle(Path::new(
                "/Applications/WarpgateSH.app/Contents/MacOS/old-name"
            ))
            .expect("bundle path"),
            Path::new("/Applications/WarpgateSH.app")
        );
        assert!(application_bundle(Path::new("/tmp/warpgatesh-companion")).is_err());
    }

    #[test]
    fn passes_paths_and_arguments_literally_outside_the_shell_script() {
        let application = Path::new("/Applications/WarpgateSH ' $sample.app");
        let argument = OsString::from("a space; $(touch unwanted)");
        let command = relaunch_command(application, 42, std::slice::from_ref(&argument));
        let arguments: Vec<_> = command.get_args().collect();
        assert_eq!(arguments[1], RELAUNCH_SCRIPT);
        assert_eq!(arguments[3], "42");
        assert_eq!(arguments[4], application);
        assert_eq!(arguments[5], argument);
        assert_eq!(command.get_current_dir(), Some(Path::new("/")));
    }
}
