use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use futures::StreamExt;
use serde::Deserialize;

use crate::common;
use crate::minecraft::version::Version;

const MINECRAFT_RESOURCES: &str = "http://resources.download.minecraft.net";

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    map_to_resources: Option<bool>,
    objects: HashMap<String, AssetIndexObject>,
    r#virtual: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AssetIndexObject {
    hash: String,
    size: u64,
}

impl Version{
    pub async fn verify_assets(&self) -> Result<(), reqwest::Error> {
        let path: PathBuf = common::join_directories(Vec::from(["assets", "indexes", &*format!("{}.json", &self.assets)])).unwrap();
        return if path.exists() {
            if path.metadata().unwrap().len() != self.asset_index.size {
                self.fetch_assets_index(&path).await
            } else {
                self.fetch_assets(&path).await
            }
        } else {
            self.fetch_assets_index(&path).await
        };
    }

    async fn fetch_assets_index(&self, path: &PathBuf) -> Result<(), reqwest::Error> {
        match common::file_downloader::from_url(&self.asset_index.url, &path).await {
            Ok(()) => return self.fetch_assets(&path).await,
            Err(e) => panic!("{}", e)
        }
    }

    async fn fetch_assets(&self, path: &PathBuf) -> Result<(), reqwest::Error> {
        let mut file = File::open(path).expect("Something");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Unable to read file");

        let asset_index: AssetIndex = serde_json::from_str(&data).expect("");
        match asset_index.r#virtual {//Todo: map to resources
            Some(_val) => {
                let mut assets_objects: HashMap<String, AssetIndexObject> = HashMap::new();
                for object in asset_index.objects {
                    let mut path_vector: Vec<&str> = Vec::from(["assets", "virtual", &self.assets]);
                    for path in object.0.split('/') {
                        path_vector.push(path);
                    }
                    let path: PathBuf = common::join_directories(path_vector).unwrap();
                    if !path.exists() || path.metadata().unwrap().len() != object.1.size {
                        assets_objects.insert(object.0, object.1);
                    }
                }

                let assets_objects_size = assets_objects.len().to_owned();
                if assets_objects_size > 0 {
                    let fetches = futures::stream::iter(
                        assets_objects.into_iter().map(|object| {
                            async move {
                                let mut path_vector: Vec<&str> = Vec::from(["assets", "virtual", &self.assets]);
                                for path in object.0.split('/') {
                                    path_vector.push(path);
                                }
                                let path: PathBuf = common::join_directories(path_vector).unwrap();
                                let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &object.1.hash[0..2], complete_hash = &object.1.hash);
                                match common::file_downloader::from_url(&url, &path).await {
                                    Ok(()) => {}
                                    Err(e) => panic!("{}", e)
                                }
                            }
                        })
                    ).buffer_unordered(assets_objects_size).collect::<Vec<()>>();//sips memory todo all buffer
                    fetches.await;
                }
            }
            None => {
                let mut assets_objects: HashMap<String, AssetIndexObject> = HashMap::new();
                for object in asset_index.objects {
                    let path: PathBuf = common::join_directories(Vec::from(["assets", "objects", &object.1.hash[0..2], &object.1.hash])).unwrap();
                    if !path.exists() || path.metadata().unwrap().len() != object.1.size {
                        assets_objects.insert(object.0, object.1);
                    }
                }

                let assets_objects_size = assets_objects.len().to_owned();
                if assets_objects_size > 1 {
                    let fetches = futures::stream::iter(
                        assets_objects.into_iter().map(|object| {
                            async move {
                                let path: PathBuf = common::join_directories(Vec::from(["assets", "objects", &object.1.hash[0..2], &object.1.hash])).unwrap();
                                let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &object.1.hash[0..2], complete_hash = &object.1.hash);
                                match common::file_downloader::from_url(&url, &path).await {
                                    Ok(()) => {}
                                    Err(e) => panic!("{}", e)
                                }
                            }
                        })
                    ).buffer_unordered(assets_objects_size).collect::<Vec<()>>();
                    fetches.await;
                }
            }
        }
        Ok(())
    }
}