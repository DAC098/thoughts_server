pub mod error;

pub fn file_from_arg(arg: &String) -> error::Result<std::path::PathBuf> {
    if let Ok(canonical_path) = std::fs::canonicalize(arg) {
        if !canonical_path.is_file() {
            Err(error::Error::InvalidFile(canonical_path.into_os_string()))
        } else {
            Ok(canonical_path)
        }
    } else {
        Err(error::Error::FileNotFound(arg.clone()))
    }
}

pub fn get_cli_option<'a>(arg: &'a String) -> error::Result<Option<&'a str>> {
    if arg.starts_with("--") {
        if arg.len() <= 2 {
            Err(error::Error::IncompleteArg)
        } else {
            let (_, substring) = arg.split_at(2);

            Ok(Some(substring))
        }
    } else {
        Ok(None)
    }
}

pub fn get_cli_option_value<N>(
    args: &mut std::env::Args, 
    name: N
) -> error::Result<String>
where
    N: AsRef<str>
{
    if let Some(next_arg) = args.next() {
        Ok(next_arg)
    } else {
        Err(error::Error::MissingArgValue(name.as_ref().to_owned()))
    }
}