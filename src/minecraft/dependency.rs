use crate::minecraft::version::{Version, VersionLibraryDownloadObject};
use std::path::PathBuf;
use crate::common;

pub async fn download_libraries(version: &Version) -> Result<(), reqwest::Error>{
    for library in &version.libraries {
        let artifact = &library.downloads.artifact;
        match artifact {
            Some(artifact) => {
                let path: PathBuf = common::join_directories(Vec::from(["libraries", &artifact.path])).unwrap();//fixme:
                if path.exists() {
                    if path.metadata().unwrap().len() != artifact.size {
                        match common::file_downloader::from_url(&artifact.url, &path).await {
                            Ok(()) => println!("Downloaded Library: {}", &library.name),
                            Err(e) => println!("{}", e)
                        }
                    }
                }else{
                    match common::file_downloader::from_url(&artifact.url, &path).await {
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
    let path: PathBuf = common::join_directories(Vec::from(["libraries", &version_library_download_object.path])).unwrap();//fixme:
    if path.exists() {
        if path.metadata().unwrap().len() != version_library_download_object.size {
            match common::file_downloader::from_url(&version_library_download_object.url, &path).await {
                Ok(()) => println!("Fetched: {}", version_library_download_object.path),
                Err(e) => println!("{}", e)
            }
        }
    }else{
        match common::file_downloader::from_url(&version_library_download_object.url, &path).await {
            Ok(()) => println!("Fetched: {}", version_library_download_object.path),
            Err(e) => println!("{}", e)
        }
    }
}