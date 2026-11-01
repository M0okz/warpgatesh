fn main() {
    #[cfg(target_os = "macos")]
    {
        if std::env::args().any(|a| a == "--recover-update") {
            if let Err(error) = warpgatesh_companion::recover() {
                eprintln!("{error}");
                std::process::exit(1);
            }
        } else if std::env::args().any(|a| a == "--native-backend") {
            warpgatesh_companion::run();
        } else {
            use std::os::unix::process::CommandExt;
            let path = std::env::current_exe()
                .expect("resolve executable")
                .with_file_name("warpgatesh-ui");
            let error = std::process::Command::new(path)
                .args(std::env::args_os().skip(1))
                .exec();
            eprintln!("WarpgateSH: {error}");
            std::process::exit(1);
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("The native companion requires macOS.");
        std::process::exit(1);
    }
}
