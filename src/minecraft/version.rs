use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

use crate::common;
use crate::minecraft::version_manifest::{VersionManifest, VersionManifestVersion};
use crate::minecraft::dependency::{LibrariesMetadataDependency, LibrariesMetadataDependencyNative, LibrariesMetadataDependencyNativeRule, Dependency};
use serde::__private::Option::Some;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub arguments: Option<VersionArgument>,
    pub asset_index: VersionAssetIndex,
    pub assets: String,
    pub compliance_level: Option<u64>,
    pub downloads: VersionDownload,
    pub id: String,
    pub libraries: Vec<VersionLibrary>,
    pub logging: Option<Value>,
    pub main_class: String,
    pub minecraft_arguments: Option<String>,
    pub minimum_launcher_version: u64,
    pub release_time: String,
    pub time: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionArgument {
    pub game: Vec<Value>,
    pub jvm: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionAssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionDownload {
    pub client: Option<VersionDownloadObject>,
    pub client_mappings: Option<VersionDownloadObject>,
    pub server: Option<VersionDownloadObject>,
    pub server_mappings: Option<VersionDownloadObject>,
}

#[derive(Debug, Deserialize)]
pub struct VersionDownloadObject {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionLibrary {
    pub downloads: Option<VersionLibraryDownload>,
    pub name: String,
    pub url: Option<String>,//maven link
    pub natives: Option<HashMap<String, String>>,
    pub rules: Option<Vec<VersionLibraryRule>>,
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryDownload {
    pub artifact: Option<VersionLibraryDownloadObject>,
    pub classifiers: Option<HashMap<String, VersionLibraryDownloadObject>>,
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryDownloadObject {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryRule{
    pub action: String,
    pub os: Option<VersionLibraryRuleOSObject>
}

#[derive(Debug, Deserialize)]
pub struct VersionLibraryRuleOSObject{
    pub name: String,
    pub version: Option<String>
}

impl Version{
    pub async fn get_version(version: &str) -> Result<Option<Version>, std::io::Error> {
        let path: PathBuf = common::join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version, &*format!("{}.json", version)])).unwrap();
        if path.exists() {
            return Version::read(&path);
        } else {
            match VersionManifest::get(true).await {
                Ok(val) => {
                    match val.versions.iter().find(|x| x.id.eq(version)) {
                        Some(ver) => {
                            match Version::fetch(ver).await {
                                Ok(()) => return Version::read(&path),
                                Err(e) => panic!("{}", e)
                            }
                        },
                        None => println!("Unable to find version")
                    }
                }
                Err(e) => panic!("{}", e)
            }
        }
        Ok(None)
    }

    fn read(path: &PathBuf) -> Result<Option<Version>, std::io::Error> {
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Unable to read file");
        let version: Version = serde_json::from_str(&data).expect("JSON was not well-formatted");
        Ok(Some(version))
    }

    pub async fn fetch(version: &VersionManifestVersion) -> Result<(), reqwest::Error> {
        let path: PathBuf = common::join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version.id, &*format!("{}.json", &version.id)])).unwrap();
        match common::file_downloader::from_url(&version.url, &path).await {
            Ok(()) => Ok(()),
            Err(e) => panic!("{}", e)
        }
    }

    pub fn get_libraries(&self) -> Vec<String>{
        let mut libraries: Vec<String> = Vec::new();
        for library in &self.libraries {
            libraries.push(library.name.to_owned());
        }
        libraries
    }

    pub fn get_game_arguments(&self) -> String{
        let mut arguments: String = String::new();
        if let Some(argument) = &self.arguments{
            for val in &argument.game {
                if val.is_string() {
                    arguments.push_str(&format!("{} ", val.as_str().unwrap()));
                }
            }
        }else if let Some(argument) = &self.minecraft_arguments{
            arguments.push_str(argument);
        }
        arguments.trim().to_string()
    }
}

impl VersionLibrary{
    pub async fn to_libraries_metadata_dependency(&self, version: &String) -> LibrariesMetadataDependency{
        let id: String = version.to_string();
        let name: String = self.name.to_owned().to_string();

        let size: Option<u64> = if let Some(downloads) = &self.downloads {
            if let Some(artifact) = &downloads.artifact {
                Some(artifact.size)
            } else {
                None
            }
        } else if let Some(url) = &self.url {
            let dependency = Dependency::from_version_library(&self);
            if let Some(dependency) = dependency {
                let url = format!("{url}/{maven_layout}", url = url, maven_layout = dependency.to_maven_url_layout_jar());
                let res = reqwest::get(&url).await;//fixme: really dumb
                match res {
                    Ok(res) => {
                        if let Some(length) = res.content_length() {
                            Some(length)
                        }else{
                            None
                        }
                    },
                    Err(e) => panic!("{}", e)
                }
            } else {
                None
            }
        } else {
            None
        };

        let url: Option<String> = if let Some(downloads) = &self.downloads {
            if let Some(artifact) = &downloads.artifact {
                Some(artifact.url.to_owned().to_string())
            } else {
                None
            }
        } else if let Some(url) = &self.url {
            let dependency = Dependency::from_version_library(&self);
            if let Some(dependency) = dependency {
                Some(format!("{url}/{maven_layout}", url = url, maven_layout = dependency.to_maven_url_layout_jar()))
            } else {
                None
            }
        } else {
            None
        };

        let path: Option<String> = if let Some(downloads) = &self.downloads {
            if let Some(artifact) = &downloads.artifact {
                Some(artifact.path.to_owned().to_string())
            } else {
                None
            }
        } else if let Some(_url) = &self.url {
            let dependency = Dependency::from_version_library(&self);
            if let Some(dependency) = dependency {
                Some(dependency.to_maven_url_layout_jar())
            } else {
                None
            }
        } else {
            None
        };

        let native: Option<LibrariesMetadataDependencyNative> = LibrariesMetadataDependencyNative::parse_from(&self);

        LibrariesMetadataDependency {id, name, size, url, path, native}
    }
}