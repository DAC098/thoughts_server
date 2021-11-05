use std::fs::{File};
use std::io::{Result, Read, Write};
use std::path::{Path, PathBuf};

pub fn get_tmp_path<P>(dir: P, ext: &str) -> PathBuf
where
    P: AsRef<Path> 
{
    let mut count = 0u16;
    let now = chrono::Utc::now().timestamp();
    let mut tmp_path = PathBuf::from(dir.as_ref());

    loop {
        tmp_path.push(format!("{:016x}{:04x}.{}", now, count, ext));

        if !tmp_path.exists() {
            return tmp_path;
        } else {
            count += 1;
            tmp_path.pop();
        }
    }
}

pub fn copy_file<P>(mut file: &File, path: P) -> Result<File>
where
    P: AsRef<Path>
{
    let mut rtn_file = File::create(path)?;
    let mut buffer = [0; 2048];

    while file.read(&mut buffer)? > 0 {
        rtn_file.write(&mut buffer)?;
    }

    Ok(rtn_file)
}