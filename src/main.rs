use std::io;
use common::minecraft::VersionManifest;
use futures::StreamExt;
use std::time::SystemTime;

mod common;

#[tokio::main]
async fn main() {
    /*let response: common::minecraft::AssetIndex = reqwest::get("https://launchermeta.mojang.com/v1/packages/d6c94fad4f7a03a8e46083c023926515fc0e551e/1.14.json")
        .await
        .expect("Something went wrong")
        .json()
        .await
        .expect("Something");
    common::minecraft::download_assets(response).await;*/

    match common::minecraft::get_version("1.16.5").unwrap() {
        Some(val) => {
            for library in &val.libraries {
                println!("{}", &library.downloads.artifact.url);
            }
        }
        None => {}
    }
}