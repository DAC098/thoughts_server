pub mod error;

pub fn init_from_cli() -> error::Result<Vec<std::path::PathBuf>> {
    let mut config_files: Vec<std::path::PathBuf> = vec!();
    let mut args = std::env::args();
    args.next();

    while let Some(arg) = args.next() {
        if arg.starts_with("--") {
            if arg.len() <= 2 {
                return Err(error::CliError::IncompleteArg);
            }

            let (_, arg_substring) = arg.split_at(2);

            if arg_substring == "log-debug" {
                std::env::set_var("RUST_LOG", "debug");
            } else if arg_substring == "log-info" {
                std::env::set_var("RUST_LOG", "info")
            } else if arg_substring == "backtrace" {
                std::env::set_var("RUST_BACKTRACE", "full");
            } else if arg_substring == "info" {
                std::env::set_var("RUST_LOG", "info");
            } else {
                return Err(error::CliError::UnknownArg(arg));
            }
        } else {
            if let Ok(canonical_path) = std::fs::canonicalize(arg.clone()) {
                if !canonical_path.is_file() {
                    return Err(error::CliError::InvalidFile(canonical_path.into_os_string()));
                }
    
                config_files.push(canonical_path);
            } else {
                return Err(error::CliError::FileNotFound(arg));
            }
        }
    }

    env_logger::init();

    Ok(config_files)
}