use serde::{Deserialize, Serialize};
use futures::StreamExt;
use crate::common::file_downloader;
use reqwest::Error;
use std::collections::HashMap;
use std::path::Path;
use std::fs::{Metadata, File};
use serde_json::Value;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct VersionManifest{
    pub latest: VersionManifestLatest,
    pub versions: Vec<VersionManifestVersion>
}

#[derive(Debug, Deserialize)]
pub struct VersionManifestLatest {
    pub release: String,
    pub snapshot: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifestVersion {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String
}

pub async fn get_version_manifest() -> Result<Option<VersionManifest>, reqwest::Error>{
    let client = reqwest::Client::new();

    let response = client.get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .send()
        .await?;

    let value: VersionManifest = response.json().await?;

    Ok(Some(value))
}

pub async fn download_version_manifest() -> Result<(), reqwest::Error>{
    match file_downloader::from_url("https://launchermeta.mojang.com/mc/game/version_manifest.json", "./run/versions/version_manifest.json").await {
        Ok(()) => {},
        Err(e) => panic!("{}", e),
    }
    Ok(())
}

pub async fn download_versions(version_manifest: VersionManifest) -> Result<(), reqwest::Error>{
    let fetches = futures::stream::iter(
        version_manifest.versions.into_iter().map(|version| {
            async move {
                let test_version = &version;
                match file_downloader::from_url(&*test_version.url, &*format!("./run/versions/{version}/{version}.json", version = test_version.id)).await {
                    Ok(()) => {}
                    Err(e) => panic!("{}", e),
                }
            }
        })
    ).buffer_unordered(100).collect::<Vec<()>>();
    fetches.await;
    Ok(())
}

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

pub async fn download_assets(asset_index: AssetIndex) -> Result<(), reqwest::Error>{

    const MINECRAFT_RESOURCES: &str = "http://resources.download.minecraft.net";

    match asset_index.map_to_resources {
        Some(val) => {
            //map to resources
        }
        None => {
            let fetches = futures::stream::iter(
                asset_index.objects.into_iter().map(|index_object| {
                    async move {
                        let index_object = &index_object.1;
                        let path = format!("./run/assets/objects/{two_hash}/{complete_hash}", two_hash = &index_object.hash[0..2], complete_hash = &index_object.hash);
                        let path_as: &Path = Path::new(&path);
                        if path_as.exists() {
                            let metadata = std::fs::metadata(&path);
                            match metadata {
                                Ok(val) => {
                                    if val.len() != index_object.size {
                                        let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &index_object.hash[0..2], complete_hash = &index_object.hash);
                                        match file_downloader::from_url(&*url, &*path).await {
                                            Ok(()) => {},
                                            Err(e) => panic!("{}", e)
                                        }
                                    }
                                }
                                Err(e) => panic!("{}", e)
                            }
                        }else{
                            let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &index_object.hash[0..2], complete_hash = &index_object.hash);
                            match file_downloader::from_url(&*url, &*path).await {
                                Ok(()) => {},
                                Err(e) => panic!("{}", e)
                            }
                        }
                    }
                })
            ).buffer_unordered(100).collect::<Vec<()>>();
            fetches.await;
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version{
    pub arguments: Option<VersionArgument>,
    pub asset_index: VersionAssetIndex,
    pub assets: String,
    pub compliance_level: u64,
    pub downloads: VersionDownload,
    pub id: String,
    pub libraries: Vec<VersionLibrary>,
    pub logging: Option<Value>, //todo:
    pub main_class: String,
    pub minecraft_arguments: Option<String>,
    pub minimum_launcher_version: u64,
    pub release_time: String,
    pub time: String,
    pub r#type: String
}

#[derive(Debug, Deserialize)]
pub struct VersionArgument{
    pub game: Vec<Value>,
    pub jvm: Vec<Value>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionAssetIndex{
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String
}

#[derive(Debug, Deserialize)]
pub struct VersionDownload{
    pub client: Option<VersionDownloadObject>,
    pub client_mappings: Option<VersionDownloadObject>,
    pub server: Option<VersionDownloadObject>,
    pub server_mappings: Option<VersionDownloadObject>,
}

#[derive(Debug, Deserialize)]
pub struct VersionDownloadObject{
    pub sha1: String,
    pub size: u64,
    pub url: String
}

#[derive(Debug, Deserialize)]
pub struct VersionLibrary{
    pub downloads: VersionLibraryDownload,
    pub name: String,
    pub rule: Option<Value>//todo:
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryDownload{
    pub artifact: VersionLibraryDownloadArtifact
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryDownloadArtifact{
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String
}

pub fn get_version(version: &str) -> Result<Option<Version>, std::io::Error>{
    let path: &str = &format!("./run/versions/{version}/{version}.json", version = version).to_owned()[..];
    let path = Path::new(path);
    if path.exists() {
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Unable to read file");

        let version: Version = serde_json::from_str(&data).expect("JSON was not well-formatted");
        return Ok(Some(version));
    }
    Ok(None)
}