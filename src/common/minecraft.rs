use serde::Deserialize;
use futures::StreamExt;
use crate::common::file_downloader;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use serde_json::Value;
use std::io::{Read, Write};
use reqwest::Error;
use std::process::Stdio;
use crate::common::auth::yggdrasil::AuthenticateResponse;
use std::ptr::replace;
use std::io;
use zip::ZipArchive;

const MINECRAFT_RESOURCES: &str = "http://resources.download.minecraft.net";

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

pub async fn get_version_manifest(refresh: bool) -> Result<VersionManifest, std::io::Error>{
    let path: PathBuf = join_directories(Vec::from(["meta", "com", "mojang", "minecraft", "version_manifest.json"])).unwrap();
    if path.exists() && !refresh {
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Unable to read file");

        let version: VersionManifest = serde_json::from_str(&data).expect("JSON was not well-formatted");
        return Ok(version);
    }else{
        match file_downloader::from_url("https://launchermeta.mojang.com/mc/game/version_manifest.json", &path).await {
            Ok(()) => {
                let mut file = File::open(path)?;
                let mut data = String::new();
                file.read_to_string(&mut data).expect("Unable to read file");

                let version: VersionManifest = serde_json::from_str(&data).expect("JSON was not well-formatted");
                return Ok(version);
            }
            Err(e) => panic!("{}", e)
        }
    }
}

pub async fn download_versions(version_manifest: VersionManifest) -> Result<(), reqwest::Error>{
    let fetches = futures::stream::iter(
        version_manifest.versions.into_iter().map(|version| {
            async move {
                let path: PathBuf = join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version.id, &*format!("{}.json", &version.id)])).unwrap();
                match file_downloader::from_url(&version.url, &path).await {
                    Ok(()) => {}
                    Err(e) => panic!("{}", e),
                }
            }
        })
    ).buffer_unordered(100).collect::<Vec<()>>();
    fetches.await;
    Ok(())
}

pub async fn download_version(version: VersionManifestVersion) -> Result<(), reqwest::Error>{
    let path: PathBuf = join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version.id, &*format!("{}.json", &version.id)])).unwrap();
    match file_downloader::from_url(&version.url, &path).await {
        Ok(()) => Ok(()),
        Err(e) => panic!("{}", e)
    }
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

pub async fn download_assets(version: &Version) -> Result<(), reqwest::Error>{
    let path: PathBuf = join_directories(Vec::from(["assets", "indexes", &*format!("{}.json", &version.asset_index.id)])).unwrap();
    if path.exists() {
        if path.metadata().unwrap().len() != version.asset_index.size {
            match file_downloader::from_url(&version.asset_index.url, &path).await {
                Ok(()) => {
                    return fetch_assets(&path).await
                },
                Err(e) => panic!("{}", e)
            }
        }else{
            return fetch_assets(&path).await;
        }
    }else{
        match file_downloader::from_url(&version.asset_index.url, &path).await {
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
                        let path: PathBuf = join_directories(Vec::from(["assets", "objects", &index_object.hash[0..2], &index_object.hash])).unwrap();
                        if path.exists() {
                            match path.metadata() {
                                Ok(val) => {
                                    if val.len() != index_object.size {
                                        let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &index_object.hash[0..2], complete_hash = &index_object.hash);
                                        match file_downloader::from_url(&url, &path).await {
                                            Ok(()) => println!("Fetched Resource: {}", object.0),
                                            Err(e) => panic!("{}", e)
                                        }
                                    }
                                }
                                Err(e) => panic!("{}", e)
                            }
                        }else{
                            let url = format!("{api}/{two_hash}/{complete_hash}", api = MINECRAFT_RESOURCES, two_hash = &index_object.hash[0..2], complete_hash = &index_object.hash);
                            match file_downloader::from_url(&url, &path).await {
                                Ok(()) => println!("Fetched Resource: {}", object.0),
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
    let path: PathBuf = join_directories(Vec::from(["meta", "com", "mojang", "minecraft", &version, &*format!("{}.json", version)])).unwrap();
    if path.exists() {
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Unable to read file");

        let version: Version = serde_json::from_str(&data).expect("JSON was not well-formatted");
        return Ok(Some(version));
    }else{
        match get_version_manifest(true).await {
            Ok(val) => {
                for ver in val.versions {
                    if ver.id.eq(version) {
                        match download_version(ver).await {
                            Ok(()) => {
                                let mut file = File::open(path)?;
                                let mut data = String::new();
                                file.read_to_string(&mut data).expect("Unable to read file");

                                let version: Version = serde_json::from_str(&data).expect("JSON was not well-formatted");
                                return Ok(Some(version));
                            },
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

pub async fn download_libraries(version: &Version) -> Result<(), reqwest::Error>{
    for library in &version.libraries {
        let artifact = &library.downloads.artifact;
        match artifact {
            Some(artifact) => {
                let path: PathBuf = join_directories(Vec::from(["libraries", &artifact.path])).unwrap();//fixme:
                if path.exists() {
                    if path.metadata().unwrap().len() != artifact.size {
                        match file_downloader::from_url(&artifact.url, &path).await {
                            Ok(()) => println!("Downloaded Library: {}", &library.name),
                            Err(e) => println!("{}", e)
                        }
                    }
                }else{
                    match file_downloader::from_url(&artifact.url, &path).await {
                        Ok(()) => println!("Downloaded Library: {}", &library.name),
                        Err(e) => println!("{}", e)
                    }
                }
            },
            None => {}
        }
    }
    Ok(())
}

pub async fn download_natives(version: &Version) -> Result<(), reqwest::Error>{
    for library in &version.libraries {
        let classifiers = &library.downloads.classifiers;
        match classifiers {
            Some(classifiers) => {
                let os = std::env::consts::OS;
                if os.eq("windows") && classifiers.contains_key("natives-windows") {
                    download_native(classifiers.get("natives-windows").unwrap()).await;
                }else if os.eq("linux") && classifiers.contains_key("natives-linux") {
                    download_native(classifiers.get("natives-linux").unwrap()).await;
                }else if os.eq("macos") && classifiers.contains_key("natives-macos") {
                    download_native(classifiers.get("natives-macos").unwrap()).await;
                }
            },
            None => {}
        }
    }
    Ok(())
}

async fn download_native(version_library_download_object: &VersionLibraryDownloadObject){
    let path: PathBuf = join_directories(Vec::from(["libraries", &version_library_download_object.path])).unwrap();//fixme:
    if path.exists() {
        if path.metadata().unwrap().len() != version_library_download_object.size {
            match file_downloader::from_url(&version_library_download_object.url, &path).await {
                Ok(()) => println!("Fetched: {}", version_library_download_object.path),
                Err(e) => println!("{}", e)
            }
        }
    }else{
        match file_downloader::from_url(&version_library_download_object.url, &path).await {
            Ok(()) => println!("Fetched: {}", version_library_download_object.path),
            Err(e) => println!("{}", e)
        }
    }
}


pub async fn download_client(version: &Version) -> Result<(), reqwest::Error>{
    let path: PathBuf = join_directories(Vec::from(["libraries", "com", "mojang", "minecraft", &version.id, &*format!("{}.jar", version.id)])).unwrap();
    if path.exists() {
        if path.metadata().unwrap().len() != version.downloads.client.as_ref().unwrap().size {
            match file_downloader::from_url(&*version.downloads.client.as_ref().unwrap().url, &path).await {
                Ok(()) => {},
                Err(e) => panic!("{}", e)
            }
        }
    }else{
        match file_downloader::from_url(&*version.downloads.client.as_ref().unwrap().url, &path).await {
            Ok(()) => {},
            Err(e) => panic!("{}", e)
        }
    }
    Ok(())
}


pub fn launch_client(authentication_response: &AuthenticateResponse, version: &Version){
    let mut jvm_arguments: Vec<String> = Vec::new();
    let mut game_arguments:Vec<&str> = Vec::new();
    let auth = authentication_response;
    match &version.arguments {
        Some(val) => {
            for jvm_argument in &val.jvm {
                if jvm_argument.is_string() {
                    let new_arg: String = {
                        let arg: &str = jvm_argument.as_str().unwrap();
                        if arg.contains("${launcher_name}") {
                            arg.replace("${launcher_name}", "DuckLauncher")
                        }else if arg.contains("${natives_directory}") {
                            //todo: extract natives
                            let mut paths: Vec<PathBuf> = Vec::new();
                            for library in &version.libraries {
                                //Extract natives
                                let classifiers = &library.downloads.classifiers;
                                match classifiers {
                                    Some(classifiers) => {
                                        let os = std::env::consts::OS;
                                        let path:Option<PathBuf> = {
                                            if os.eq("windows") && classifiers.contains_key("natives-windows") {
                                                Some(join_directories(Vec::from(["libraries", &classifiers.get("natives-windows").unwrap().path])).unwrap())
                                            }else if os.eq("linux") && classifiers.contains_key("natives-linux") {
                                                Some(join_directories(Vec::from(["libraries", &classifiers.get("natives-linux").unwrap().path])).unwrap())
                                            }else if os.eq("macos") && classifiers.contains_key("natives-macos") {
                                                Some(join_directories(Vec::from(["libraries", &classifiers.get("natives-macos").unwrap().path])).unwrap())
                                            }else{
                                                None
                                            }
                                        };
                                        match path {
                                            Some(path) => paths.push(path),
                                            None => {}
                                        }
                                    },
                                    None => {}
                                }
                            }

                            let natives_path = join_directories(Vec::from(["instances", &version.id, "natives"])).unwrap();
                            for path in paths {
                                let file = File::open(path).expect("");
                                let mut zip = ZipArchive::new(file).expect("");
                                zip.extract(&natives_path);
                            }

                            arg.replace("${natives_directory}", &natives_path.into_os_string().into_string().unwrap())
                        }else if arg.contains("${launcher_version}") {
                            arg.replace("${launcher_version}", "1")
                        }else if arg.contains("${classpath}") {
                            let mut paths: Vec<String> = Vec::new();

                            for library in &version.libraries {
                                let artifact = &library.downloads.artifact;
                                match artifact {
                                    Some(artifact) => {
                                        let path: PathBuf = join_directories(Vec::from(["libraries", &artifact.path])).unwrap();
                                        paths.push(path.into_os_string().into_string().unwrap());
                                    },
                                    None => {}
                                }

                                let path: PathBuf = join_directories(Vec::from(["libraries", "com", "mojang", "minecraft", &version.id, &*format!("{}.jar", version.id)])).unwrap();
                                paths.push(path.into_os_string().into_string().unwrap())
                            }

                            let mut builder: String = String::new();
                            for path in paths {
                                if builder.is_empty() {
                                    builder.push_str(&path);
                                }else{
                                    builder.push_str(&format!(";{}", path));
                                }
                            }
                            arg.replace("${classpath}", builder.as_str())
                        }else{
                            arg.to_string()
                        }
                    };
                    jvm_arguments.push(new_arg);
                }
            }


            for game_argument in &val.game {
                if game_argument.is_string() {
                    let arg: &str = game_argument.as_str().unwrap();
                    if arg.contains("${") && arg.contains("}") {
                        if arg.contains("auth_player_name") {
                            game_arguments.push(&*auth.selected_profile.as_ref().unwrap().name.as_ref().unwrap());
                        }
                        if arg.contains("version_name") {
                            game_arguments.push(&*version.id);
                        }
                        if arg.contains("game_directory") {
                            game_arguments.push("D:\\Projects\\Intellij Workspace\\duck-launcher\\run\\instances\\1.16.5\\.minecraft");
                        }
                        if arg.contains("assets_root") {
                            game_arguments.push("D:\\Projects\\Intellij Workspace\\duck-launcher\\run\\assets");
                        }
                        if arg.contains("assets_index_name") {
                            game_arguments.push(&*version.asset_index.id);
                        }
                        if arg.contains("auth_uuid") {
                            game_arguments.push(&*auth.selected_profile.as_ref().unwrap().id.as_ref().unwrap());
                        }
                        if arg.contains("auth_access_token") {
                            game_arguments.push(&*auth.access_token.as_ref().unwrap());
                        }
                        if arg.contains("user_type") {
                            game_arguments.push("mojang");
                        }
                        if arg.contains("version_type") {
                            game_arguments.push(&*version.r#type);
                        }
                    }else{
                        game_arguments.push(arg);
                    }
                }
            }
        },
        None => {}
    }
    let mut command = std::process::Command::new("java");
    for jvm_argument in jvm_arguments {
        command.arg(&*jvm_argument);
        println!("{}", &*jvm_argument);
    }
    command.arg(&version.main_class);
    for argument in game_arguments {
        command.arg(argument);
        println!("{}", argument);
    }
    println!("Starting Client");
    let output = command.output().expect("kekw");
    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    /*if ! {
        println!("kekw");
    }*/
}


fn join_directories(vec: Vec<&str>) -> std::io::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    for s in vec {
        dir.push(s);
    }
    Ok(dir)
}