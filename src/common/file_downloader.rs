use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub async fn from_url(url: &str, path: &PathBuf) -> Result<(), reqwest::Error> {
    let path_str: String = path.clone().into_os_string().into_string().unwrap();
    let response = reqwest::get(url).await?;

    println!("Downloading {url} to {path}", url = url, path = path_str);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let mut file = match File::create(path) {
        Err(why) => panic!("Couldn't Create {}", why),
        Ok(file) => file,
    };
    let content = response.bytes().await?;
    match file.write_all(&*content) {
        Err(e) => println!("{}", e),
        Ok(()) => println!("Downloaded {url} to {path}", url = url, path = path_str)
    };

    Ok(())
}