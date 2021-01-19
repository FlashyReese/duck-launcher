use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

pub async fn from_url(url: &str, path: &PathBuf) -> Result<(), reqwest::Error> {
    let response = reqwest::get(url).await?;

    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };
    let content =  response.bytes().await?;
    match file.write_all(&*content) {
        Err(e) => println!("{}", e),
        _ => {}
    };

    Ok(())
}