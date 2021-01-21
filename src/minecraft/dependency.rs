use std::collections::HashMap;
use std::path::PathBuf;

use futures::StreamExt;

use crate::common;
use crate::minecraft::version::{Version, VersionLibrary, VersionLibraryDownloadObject};

pub struct Dependency {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

fn parse_to_dependency(version_library: &VersionLibrary) -> Option<Dependency> {
    let split = version_library.name.split(':').collect::<Vec<&str>>();
    if split.len() != 3 {
        None
    } else {
        let group = {
            match split.get(0) {
                Some(val) => val.to_owned().to_string(),
                None => return None
            }
        };
        let artifact = {
            match split.get(1) {
                Some(val) => val.to_owned().to_string(),
                None => return None
            }
        };
        let version = {
            match split.get(2) {
                Some(val) => val.to_owned().to_string(),
                None => return None
            }
        };
        let dependency = Dependency {
            group,
            artifact,
            version,
        };
        Some(dependency)
    }
}

pub fn get_required_libraries(version: &Version) -> Option<HashMap<&str, (Dependency, &VersionLibrary)>> {
    let mut dependencies: HashMap<&str, (Dependency, &VersionLibrary)> = HashMap::new();
    for library in &version.libraries {
        let dependency_library = parse_to_dependency(&library).unwrap();
        dependencies.insert(&library.name, (dependency_library, library));
    }

    let mut remove_dependencies: Vec<&str> = Vec::new();
    for dependency_upper in &dependencies {
        for dependency_lower in &dependencies {
            if dependency_upper.1.0.group.eq(&dependency_lower.1.0.group) && dependency_upper.1.0.artifact.eq(&dependency_lower.1.0.artifact) {
                let dep_up = semver::Version::parse(&dependency_upper.1.0.version);
                let dep_low = semver::Version::parse(&dependency_lower.1.0.version);
                if dep_up > dep_low {
                    if !remove_dependencies.contains(&dependency_lower.0) {
                        remove_dependencies.push(&dependency_lower.0)
                    }
                }
            }
        }
    }
    for remove_dependency in remove_dependencies {
        dependencies.remove(remove_dependency);
    }
    Some(dependencies)
}

pub async fn verify_libraries(version: &Version) -> Result<(), reqwest::Error> {
    match get_required_libraries(version) {
        Some(dependencies) => {
            let mut future_libraries: Vec<&VersionLibrary> = Vec::new();
            for dependency in dependencies {
                let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                for path in dependency.1.1.downloads.artifact.as_ref().unwrap().path.split('/') {
                    path_vector.push(path);
                }
                let path: PathBuf = common::join_directories(path_vector).unwrap();
                if !path.exists() || path.metadata().unwrap().len() != dependency.1.1.downloads.artifact.as_ref().unwrap().size {
                    future_libraries.push(dependency.1.1);
                }
            }

            let future_libraries_size = future_libraries.len().to_owned();
            if future_libraries_size > 0 {
                let fetches = futures::stream::iter(
                    future_libraries.into_iter().map(|library| {
                        async move {
                            let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                            for path in library.downloads.artifact.as_ref().unwrap().path.split('/') {
                                path_vector.push(path);
                            }
                            let path: PathBuf = common::join_directories(path_vector).unwrap();
                            match common::file_downloader::from_url(&library.downloads.artifact.as_ref().unwrap().url, &path).await {
                                Ok(()) => {}
                                Err(e) => panic!("{}", e)
                            }
                        }
                    })
                ).buffer_unordered(future_libraries_size).collect::<Vec<()>>();
                fetches.await;
            }
        }
        None => {}
    }
    Ok(())
}

pub fn get_required_natives(version: &Version) -> Option<Vec<&VersionLibraryDownloadObject>> {
    match get_required_libraries(version) {
        Some(dependencies) => {
            let mut natives: Vec<&VersionLibraryDownloadObject> = Vec::new();
            for dependency in dependencies {
                let classifiers = &dependency.1.1.downloads.classifiers;
                match classifiers {
                    Some(classifiers) => {
                        let os = std::env::consts::OS;
                        if os.eq("windows") && classifiers.contains_key("natives-windows") {
                            natives.push(classifiers.get("natives-windows").unwrap());
                        } else if os.eq("linux") && classifiers.contains_key("natives-linux") {
                            natives.push(classifiers.get("natives-linux").unwrap());
                        } else if os.eq("macos") && classifiers.contains_key("natives-macos") {
                            natives.push(classifiers.get("natives-macos").unwrap());
                        }
                    },
                    None => {}
                }
            }
            return Some(natives);
        },
        None => {}
    }
    None
}

pub async fn verify_natives(version: &Version) -> Result<(), reqwest::Error> {
    match get_required_natives(version) {
        Some(natives) => {
            let mut future_natives: Vec<&VersionLibraryDownloadObject> = Vec::new();
            for native in natives {
                let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                for path in native.path.split('/') {
                    path_vector.push(path);
                }
                let path: PathBuf = common::join_directories(path_vector).unwrap();
                if !path.exists() || path.metadata().unwrap().len() != native.size{
                    future_natives.push(native);
                }
            }

            let future_natives_size = future_natives.to_owned().len();
            if future_natives_size > 0 {
                let fetches = futures::stream::iter(
                    future_natives.into_iter().map(|native| {
                        async move {
                            let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                            for path in native.path.split('/') {
                                path_vector.push(path);
                            }
                            let path: PathBuf = common::join_directories(path_vector).unwrap();
                            match common::file_downloader::from_url(&native.url, &path).await {
                                Ok(()) => {}
                                Err(e) => panic!("{}", e)
                            }
                        }
                    })
                ).buffer_unordered(future_natives_size).collect::<Vec<()>>();
                fetches.await;
            }
        },
        None => {}
    }
    Ok(())
}