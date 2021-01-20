use std::path::PathBuf;

pub mod file_downloader;

pub fn join_directories(vec: Vec<&str>) -> std::io::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    for s in vec {
        dir.push(s);
    }
    Ok(dir)
}