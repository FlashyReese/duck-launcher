use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use crate::minecraft::version_manifest::{get_version_manifest, fetch_version_metadata};
use crate::common;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version{
    pub arguments: Option<VersionArgument>,
    pub asset_index: VersionAssetIndex,
    pub assets: String,
    pub compliance_level: Option<u64>,
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
    pub natives: Option<HashMap<String, String>>,
    pub rule: Option<Value>//todo:
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryDownload{
    pub artifact: Option<VersionLibraryDownloadObject>,
    pub classifiers: Option<HashMap<String, VersionLibraryDownloadObject>>
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryDownloadObject {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String
}

pub async fn get_version(version: &str) -> Result<Option<Version>, std::io::Error>{
    let path: PathBuf = common::join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version, &*format!("{}.json", version)])).unwrap();
    if path.exists() {
        return read_version_metadata(&path)
    }else{
        match get_version_manifest(true).await {
            Ok(val) => {
                for ver in val.versions {
                    if ver.id.eq(version) {
                        match fetch_version_metadata(ver).await {
                            Ok(()) => return read_version_metadata(&path),
                            Err(e) => panic!("{}", e)
                        }
                    }
                }
            }
            Err(e) => panic!("{}", e)
        }
    }
    Ok(None)
}

fn read_version_metadata(path: &PathBuf) -> Result<Option<Version>, std::io::Error>{
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");
    let version: Version = serde_json::from_str(&data).expect("JSON was not well-formatted");
    return Ok(Some(version));
}