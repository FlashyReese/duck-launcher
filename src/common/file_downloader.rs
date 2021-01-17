use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

pub async fn from_url(url: &str, path: &str) -> Result<(), reqwest::Error> {
    let response = reqwest::get(url).await?;

    let mut file = match File::create(Path::new(path)) {
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