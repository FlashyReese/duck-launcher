use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde::__private::Option::Some;

use crate::common;
use crate::minecraft::version::{Version, VersionLibrary, VersionLibraryDownloadObject};

pub struct Dependency {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

impl Dependency {
    pub fn from(version_library: &VersionLibrary) -> Option<Dependency> {
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

    pub fn to_maven_url_layout_jar(&self) -> String {
        format!("{group}/{artifact}/{version}/{artifact}-{version}.jar", group = self.group.replace('.', "/"), artifact = self.artifact, version = self.version)
    }
}

impl Version {
    pub async fn verify_libraries(&self) -> Result<(), reqwest::Error> {
        if let Some(dependencies) = self.get_required_libraries() {
            let mut future_libraries: Vec<&VersionLibrary> = Vec::new();
            for dependency in dependencies {
                let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                if let Some(artifact) = dependency.1.1.downloads.as_ref().unwrap().artifact.as_ref() {
                    for path in artifact.path.split('/') {
                        path_vector.push(path);
                    }
                }
                let path: PathBuf = common::join_directories(path_vector).unwrap();
                if let Some(artifact) = dependency.1.1.downloads.as_ref().unwrap().artifact.as_ref() {
                    if !path.exists() || path.metadata().unwrap().len() != artifact.size {
                        future_libraries.push(dependency.1.1);
                    }
                }
            }

            let future_libraries_size = future_libraries.len().to_owned();
            if future_libraries_size > 0 {
                let fetches = futures::stream::iter(
                    future_libraries.into_iter().map(|library| {
                        async move {
                            let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                            for path in library.downloads.as_ref().unwrap().artifact.as_ref().unwrap().path.split('/') {
                                path_vector.push(path);
                            }
                            let path: PathBuf = common::join_directories(path_vector).unwrap();
                            match common::file_downloader::from_url(&library.downloads.as_ref().unwrap().artifact.as_ref().unwrap().url, &path).await {
                                Ok(()) => {}
                                Err(e) => panic!("{}", e)
                            }
                        }
                    })
                ).buffer_unordered(future_libraries_size).collect::<Vec<()>>();
                fetches.await;
            }
        }
        Ok(())
    }

    pub fn get_required_libraries(&self) -> Option<HashMap<&str, (Dependency, &VersionLibrary)>> {
        let mut dependencies: HashMap<&str, (Dependency, &VersionLibrary)> = HashMap::new();
        for library in &self.libraries {
            let dependency_library = Dependency::from(&library).unwrap();
            dependencies.insert(&library.name, (dependency_library, library));
        }

        let mut remove_dependencies: Vec<&str> = Vec::new();
        for dependency_upper in &dependencies {
            for dependency_lower in &dependencies {
                if dependency_upper.1.0.group.eq(&dependency_lower.1.0.group) && dependency_upper.1.0.artifact.eq(&dependency_lower.1.0.artifact) {
                    let dep_up = semver::Version::parse(&dependency_upper.1.0.version);
                    let dep_low = semver::Version::parse(&dependency_lower.1.0.version);
                    if dep_up > dep_low && !remove_dependencies.contains(&dependency_lower.0) {
                        remove_dependencies.push(&dependency_lower.0)
                    }
                }
            }
        }
        for remove_dependency in remove_dependencies {
            dependencies.remove(remove_dependency);
        }
        Some(dependencies)
    }

    pub fn get_required_natives(&self) -> Option<Vec<&VersionLibraryDownloadObject>> {
        if let Some(dependencies) = self.get_required_libraries() {
            let mut natives: Vec<&VersionLibraryDownloadObject> = Vec::new();
            for dependency in dependencies {
                let classifiers = &dependency.1.1.downloads.as_ref().unwrap().classifiers;
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
                    }
                    None => {}
                }
            }
            return Some(natives);
        }
        None
    }

    pub async fn verify_natives(&self) -> Result<(), reqwest::Error> {
        if let Some(natives) = self.get_required_natives() {
            let mut future_natives: Vec<&VersionLibraryDownloadObject> = Vec::new();
            for native in natives {
                let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                for path in native.path.split('/') {
                    path_vector.push(path);
                }
                let path: PathBuf = common::join_directories(path_vector).unwrap();
                if !path.exists() || path.metadata().unwrap().len() != native.size {
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
        }
        Ok(())
    }

    pub fn get_required_natives_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();
        if let Some(natives) = self.get_required_natives() {
            for native in natives {
                let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                for path in native.path.split('/') {
                    path_vector.push(path);
                }
                let path: PathBuf = common::join_directories(path_vector).unwrap();
                paths.push(path);
            }
        }
        paths
    }

    pub fn get_required_libraries_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();
        if let Some(library_map) = self.get_required_libraries() {
            for library in library_map {
                let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                for path in library.1.1.downloads.as_ref().unwrap().artifact.as_ref().unwrap().path.split('/') {
                    path_vector.push(path);
                }
                let path: PathBuf = common::join_directories(path_vector).unwrap();
                paths.push(path);
            }
        }
        paths
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibrariesMetadata {
    pub version: u8,
    pub libraries: Vec<LibrariesMetadataDependency>,
    pub clients: Vec<LibrariesMetadataDependency>,
    pub servers: Vec<LibrariesMetadataDependency>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibrariesMetadataDependency {
    pub id: String,
    pub name: String,
    pub url: String,
    pub path: String,
    pub native: LibrariesMetadataDependencyNative,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibrariesMetadataDependencyNative {
    pub platforms: HashMap<String, LibrariesMetadataDependency>,
    pub rules: HashMap<String, bool>,
}

impl LibrariesMetadata {
    pub fn new() -> LibrariesMetadata {
        let path: PathBuf = common::join_directories(Vec::from(["libraries", "libraries_metadata.json"])).unwrap();
        if path.exists() {
            let mut file = File::open(path).expect("");
            let mut data = String::new();
            file.read_to_string(&mut data).expect("Unable to read file");
            let metadata: LibrariesMetadata = serde_json::from_str(&data).expect("JSON was not well-formatted");
            metadata
        } else {
            let metadata = LibrariesMetadata {
                version: 1,
                libraries: vec![],
                clients: vec![],
                servers: vec![],
            };
            let file = File::create(path).expect("Unable to create file");
            serde_json::to_writer(file, &metadata).expect("Unable to write to file");
            metadata
        }
    }

    pub fn push_mc_version(mut self, version: &Version) -> LibrariesMetadata {
        for library in &version.libraries {
            let id = &library.name.to_owned();

            let name = &library.name.to_owned();

            let url: String = if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    artifact.url.to_owned().to_string()
                } else {
                    "kekw".to_string()//fixme:
                }
            } else if let Some(url) = &library.url {
                let dependency = Dependency::from(&library);
                if let Some(dependency) = dependency {
                    format!("{url}/{maven_layout}", url = url, maven_layout = dependency.to_maven_url_layout_jar())
                } else {
                    "kekw".to_string()
                }
            }else {
                "kekw".to_string()
            };

            let path: String = if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    artifact.path.to_owned().to_string()
                } else {
                    "kekw".to_string()//fixme:
                }
            } else if let Some(url) = &library.url {
                let dependency = Dependency::from(&library);
                if let Some(dependency) = dependency {
                    dependency.to_maven_url_layout_jar()
                } else {
                    "kekw".to_string()
                }
            }else {
                "kekw".to_string()
            };
        }
        self
    }

    pub fn push_library(&mut self, library: LibrariesMetadataDependency) {
        self.libraries.push(library);
    }

    pub fn push_client(&mut self, client: LibrariesMetadataDependency) {
        self.clients.push(client);
    }

    pub fn push_server(&mut self, server: LibrariesMetadataDependency) {
        self.servers.push(server);
    }

    pub fn save(self) {
        let path: PathBuf = common::join_directories(Vec::from(["libraries", "libraries_metadata.json"])).unwrap();
        let file = File::create(path).expect("Unable to create file");
        serde_json::to_writer(file, &self).expect("Unable to write to file");
    }
}