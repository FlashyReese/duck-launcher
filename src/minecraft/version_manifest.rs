use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;

use crate::common;

const MINECRAFT_VERSION_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub latest: VersionManifestLatest,
    pub versions: Vec<VersionManifestVersion>,
}

#[derive(Debug, Deserialize)]
pub struct VersionManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifestVersion {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
}

pub async fn get_version_manifest(refresh: bool) -> Result<VersionManifest, std::io::Error> {
    let path: PathBuf = common::join_directories(Vec::from(["meta", "com", "mojang", "minecraft", "version_manifest.json"])).unwrap();
    if path.exists() && !refresh {
        return read_version_manifest(&path);
    } else {
        match common::file_downloader::from_url(MINECRAFT_VERSION_MANIFEST, &path).await {
            Ok(()) => return read_version_manifest(&path),
            Err(e) => panic!("{}", e)
        }
    }
}

fn read_version_manifest(path: &PathBuf) -> Result<VersionManifest, std::io::Error> {
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");
    let version: VersionManifest = serde_json::from_str(&data).expect("JSON was not well-formatted");
    Ok(version)
}

pub async fn fetch_version_metadata(version: VersionManifestVersion) -> Result<(), reqwest::Error> {
    let path: PathBuf = common::join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version.id, &*format!("{}.json", &version.id)])).unwrap();
    match common::file_downloader::from_url(&version.url, &path).await {
        Ok(()) => Ok(()),
        Err(e) => panic!("{}", e)
    }
}


/*pub async fn _download_versions(version_manifest: VersionManifest) -> Result<(), reqwest::Error>{
    let fetches = futures::stream::iter(
        version_manifest.versions.into_iter().map(|version| {
            async move {
                let path: PathBuf = common::join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version.id, &*format!("{}.json", &version.id)])).unwrap();
                match common::file_downloader::from_url(&version.url, &path).await {
                    Ok(()) => {}
                    Err(e) => panic!("{}", e),
                }
            }
        })
    ).buffer_unordered(100).collect::<Vec<()>>();
    fetches.await;
    Ok(())
}*/