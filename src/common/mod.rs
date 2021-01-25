use std::path::PathBuf;
use regex::Regex;
use std::borrow::Cow;

pub mod file_downloader;

pub fn join_directories(vec: Vec<&str>) -> std::io::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    for s in vec {
        dir.push(s);
    }
    Ok(dir)
}

pub fn remove_invalid_characters(string: &str) -> String {
    let re = Regex::new(r#"[\\/:*?"<>|]"#).unwrap();
    let result = re.replace_all(string, "");
    result.to_string()
}