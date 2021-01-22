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
    pub fn from_version_library(version_library: &VersionLibrary) -> Option<Dependency> {
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

    pub fn from_version_library_download_object(version_library: &VersionLibraryDownloadObject) -> Dependency {
        let split = version_library.path.split("/").collect::<Vec<&str>>();
        let version = split.get(split.len() - 2).unwrap();
        let artifact = split.get(split.len() - 3).unwrap();
        let group = version_library.path.split(format!("/{}/{}", artifact, version).as_str()).collect::<Vec<&str>>().get(0).unwrap().to_owned().replace("/", ".");

        Dependency {
            group,
            artifact: artifact.to_string(),
            version: version.to_string(),
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
            let dependency_library = Dependency::from_version_library(&library).unwrap();
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub native: Option<LibrariesMetadataDependencyNative>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibrariesMetadataDependencyNative {
    pub platforms: HashMap<String, LibrariesMetadataDependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub rules: Option<HashMap<String, LibrariesMetadataDependencyNativeRule>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibrariesMetadataDependencyNativeRule{
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub version: Option<String>,
    pub allowed: bool
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

    pub async fn push_mc_version(mut self, version: &Version) -> LibrariesMetadata {
        for library in &version.libraries {
            let id = &library.name.to_owned();
            if self.libraries_contains(&id) {
                continue;
            }

            let name = &library.name.to_owned();

            let size: Option<u64> = if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    Some(artifact.size)
                } else {
                    None
                }
            } else if let Some(url) = &library.url {
                let dependency = Dependency::from_version_library(&library);
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

            let url: Option<String> = if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    Some(artifact.url.to_owned().to_string())
                } else {
                    None
                }
            } else if let Some(url) = &library.url {
                let dependency = Dependency::from_version_library(&library);
                if let Some(dependency) = dependency {
                    Some(format!("{url}/{maven_layout}", url = url, maven_layout = dependency.to_maven_url_layout_jar()))
                } else {
                    None
                }
            } else {
                None
            };

            let path: Option<String> = if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    Some(artifact.path.to_owned().to_string())
                } else {
                    None
                }
            } else if let Some(url) = &library.url {
                let dependency = Dependency::from_version_library(&library);
                if let Some(dependency) = dependency {
                    Some(dependency.to_maven_url_layout_jar())
                } else {
                    None
                }
            } else {
                None
            };

            let native: Option<LibrariesMetadataDependencyNative> = {
                if let Some(downloads) = &library.downloads {
                    if let Some(classifiers) = &downloads.classifiers {
                        let mut map: HashMap<String, LibrariesMetadataDependency> = HashMap::new();
                        for classifier in classifiers {
                            let dep = Dependency::from_version_library_download_object(classifier.1);
                            let id = format!("{}:{}:{}", dep.group, dep.artifact, dep.version);
                            let name = id.to_owned();
                            let size = classifier.1.size.to_owned();
                            let url = &classifier.1.url.to_owned();
                            let path = &classifier.1.path.to_owned();
                            map.insert(classifier.0.to_string(), LibrariesMetadataDependency{
                                id,
                                name,
                                size: Some(size),
                                url: Some(url.to_string()),
                                path: Some(path.to_string()),
                                native: None
                            });
                        }

                        let mut rules_map: HashMap<String, LibrariesMetadataDependencyNativeRule> = HashMap::new();
                        if let Some(rules) = &library.rules {
                            for rule in rules {
                                let mut key;
                                let allowed = rule.action.eq("allowed");
                                let mut version;
                                if let Some(os) = &rule.os{
                                    if let Some(os_version) = &os.version{
                                        version = Some(os_version.to_string());
                                    }else{
                                        version = None;
                                    }
                                    key = &os.name;
                                }else{
                                    continue;
                                }
                                rules_map.insert(key.to_string(), LibrariesMetadataDependencyNativeRule{allowed, version});
                            }
                        }
                        if rules_map.len() == 0 {
                            Some(LibrariesMetadataDependencyNative{platforms: map, rules: None})
                        }else{
                            Some(LibrariesMetadataDependencyNative{platforms: map, rules: Some(rules_map)})
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            self.push_library(LibrariesMetadataDependency {
                id: id.to_string(),
                name: name.to_string(),
                size,
                url,
                path,
                native,
            });
        }
        if let Some(client) = &version.downloads.client{
            let id = format!("com.mojang:minecraft:{}", version.id);
            if !self.clients_contains(&id) {
                let name = format!("Minecraft {}", version.id);
                let size = &client.size;
                let url = &client.url;
                let path = format!("com/mojang/minecraft/{version}/{version}-client.jar", version = version.id);
                self.push_client(LibrariesMetadataDependency{
                    id,
                    name,
                    size: Some(size.to_owned()),
                    url: Some(url.to_string()),
                    path: Some(path),
                    native: None
                });
            }
        }
        if let Some(server) = &version.downloads.server{
            let id = format!("com.mojang:minecraft:{}", version.id);
            if !self.servers_contains(&id) {
                let name = format!("Minecraft {}", version.id);
                let size = &server.size;
                let url = &server.url;
                let path = format!("com/mojang/minecraft/{version}/{version}-server.jar", version = version.id);
                self.push_server(LibrariesMetadataDependency{
                    id,
                    name,
                    size: Some(size.to_owned()),
                    url: Some(url.to_string()),
                    path: Some(path),
                    native: None
                });
            }
        }
        self
    }

    pub fn libraries_contains(&self, library_id: &String) -> bool{
        for library in &self.libraries {
            if library.id.eq(library_id) {
                return true;
            }
        }
        false
    }

    pub fn clients_contains(&self, library_id: &String) -> bool{
        for library in &self.clients {
            if library.id.eq(library_id) {
                return true;
            }
        }
        false
    }

    pub fn servers_contains(&self, library_id: &String) -> bool{
        for library in &self.servers {
            if library.id.eq(library_id) {
                return true;
            }
        }
        false
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