use std::collections::HashMap;
use crate::minecraft::version::Version;
use crate::common;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use futures::StreamExt;
use serde::Deserialize;

const MINECRAFT_RESOURCES: &str = "http://resources.download.minecraft.net";

#[derive(Debug, Deserialize)]
pub struct AssetIndex{
    map_to_resources: Option<bool>,
    objects: HashMap<String, AssetIndexObject>
}

#[derive(Debug, Deserialize)]
struct AssetIndexObject{
    hash: String,
    size: u64,
}

pub async fn download_assets(version: &Version) -> Result<(), reqwest::Error>{
    let path: PathBuf = common::join_directories(Vec::from(["assets", "indexes", &*format!("{}.json", &version.asset_index.id)])).unwrap();
    if path.exists() {
        if path.metadata().unwrap().len() != version.asset_index.size {
            match common::file_downloader::from_url(&version.asset_index.url, &path).await {
                Ok(()) => return fetch_assets(&path).await,
                Err(e) => panic!("{}", e)
            }
        }else{
            return fetch_assets(&path).await;
        }
    }else{
        match common::file_downloader::from_url(&version.asset_index.url, &path).await {
            Ok(()) => return fetch_assets(&path).await,
            Err(e) => panic!("{}", e)
        }
    }
}

async fn fetch_assets(path: &PathBuf) -> Result<(), reqwest::Error>{
    let mut file = File::open(path).expect("Something");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");

    let asset_index: AssetIndex = serde_json::from_str(&data).expect("");
    match asset_index.map_to_resources {
        Some(_val) => {
            //Todo: map to resources
        }
        None => {
            let fetches = futures::stream::iter(
                asset_index.objects.into_iter().map(|object| {
                    async move {
                        let index_object = &object.1;
                        let path: PathBuf = common::join_directories(Vec::from(["assets", "objects", &index_object.hash[0..2], &index_object.hash])).unwrap();
                        if path.exists() {
                            match path.metadata() {
                                Ok(val) => {
                                    if val.len() != index_object.size {
                                        let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &index_object.hash[0..2], complete_hash = &index_object.hash);
                                        match common::file_downloader::from_url(&url, &path).await {
                                            Ok(()) => println!("Fetched Resource: {}", object.0),
                                            Err(e) => panic!("{}", e)
                                        }
                                    }
                                }
                                Err(e) => panic!("{}", e)
                            }
                        }else{
                            let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &index_object.hash[0..2], complete_hash = &index_object.hash);
                            match common::file_downloader::from_url(&url, &path).await {
                                Ok(()) => println!("Fetched Resource: {}", object.0),
                                Err(e) => panic!("{}", e)
                            }
                        }
                    }
                })
            ).buffer_unordered(3000).collect::<Vec<()>>();
            fetches.await;
        }
    }
    Ok(())
}