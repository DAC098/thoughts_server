use std::fs::{File, OpenOptions};
use std::io::{Result, Read, Write as _};
use std::path::{Path, PathBuf};
use std::fmt::Write as _;

/// creates a tmp file path with a timestamp and count
/// 
/// the current timestamp (in seconds) is retrieved and will count up if a
/// file already exists by that name. file basename will be in this format
/// {timestamp}_{count}.{ext}
pub fn get_tmp_path<P>(dir: P, ext: &str) -> Result<PathBuf>
where
    P: AsRef<Path>
{
    let mut count = 0u16;
    let now = chrono::Utc::now()
        .timestamp();
    let mut tmp_path = PathBuf::from(dir.as_ref());
    let mut basename = String::new();

    loop {
        // is it possible for this to actually error?
        write!(&mut basename, "{}_{}.{}", now, count, ext)
            .unwrap();

        tmp_path.push(&basename);

        if !tmp_path.try_exists()? {
            return Ok(tmp_path);
        } else {
            count += 1;
            tmp_path.pop();
            basename.clear();
        }
    }
}

/// copies a given file to the desired path
pub fn copy_file<P>(mut file: &File, path: P) -> Result<File>
where
    P: AsRef<Path>
{
    let mut rtn_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    { // should something different be happening here
        let mut buffer = [0; 2048];

        while file.read(&mut buffer)? > 0 {
            rtn_file.write(&mut buffer)?;
            buffer.fill(0);
        }
    }

    Ok(rtn_file)
}