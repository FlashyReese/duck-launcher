use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use zip::ZipArchive;

use crate::common;
use crate::minecraft::version::Version;
use crate::minecraft::yggdrasil::AuthenticateResponse;
use crate::minecraft::dependency::{get_required_natives, get_required_libraries};

pub mod asset;
pub mod dependency;
pub mod version;
pub mod version_manifest;
pub mod yggdrasil;

pub async fn download_client(version: &Version) -> Result<(), reqwest::Error> {
    let path: PathBuf = common::join_directories(Vec::from(["libraries", "com", "mojang", "minecraft", &version.id, &*format!("{}.jar", version.id)])).unwrap();
    if path.exists() {
        if path.metadata().unwrap().len() != version.downloads.client.as_ref().unwrap().size {
            match common::file_downloader::from_url(&*version.downloads.client.as_ref().unwrap().url, &path).await {
                Ok(()) => {}
                Err(e) => panic!("{}", e)
            }
        }
    } else {
        match common::file_downloader::from_url(&*version.downloads.client.as_ref().unwrap().url, &path).await {
            Ok(()) => {}
            Err(e) => panic!("{}", e)
        }
    }
    Ok(())
}

pub fn launch_client(authentication_response: &AuthenticateResponse, version: &Version) {
    let mut jvm_arguments: Vec<String> = Vec::new();
    let mut game_arguments: Vec<String> = Vec::new();
    let auth = authentication_response;
    match &version.arguments {
        Some(val) => {
            for jvm_argument in &val.jvm {
                if jvm_argument.is_string() {
                    let new_arg: String = {
                        let arg: &str = jvm_argument.as_str().unwrap();
                        if arg.contains("${launcher_name}") {
                            arg.replace("${launcher_name}", "DuckLauncher")
                        } else if arg.contains("${natives_directory}") {
                            //todo: extract natives
                            let mut paths: Vec<PathBuf> = Vec::new();
                            match get_required_natives(&version) {
                                Some(natives) => {
                                    for native in natives {
                                        let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                                        for path in native.path.split('/') {
                                            path_vector.push(path);
                                        }
                                        let path: PathBuf = common::join_directories(path_vector).unwrap();
                                        paths.push(path);
                                    }
                                },
                                None => {}
                            }
                            let natives_path = common::join_directories(Vec::from(["instances", &version.id, "natives"])).unwrap();
                            for path in paths {
                                let file = File::open(path).expect("");
                                let mut zip = ZipArchive::new(file).expect("");
                                zip.extract(&natives_path);
                            }

                            arg.replace("${natives_directory}", &natives_path.into_os_string().into_string().unwrap())
                        } else if arg.contains("${launcher_version}") {
                            arg.replace("${launcher_version}", "1")
                        } else if arg.contains("${classpath}") {
                            let mut paths: Vec<PathBuf> = Vec::new();

                            match get_required_libraries(&version) {
                                Some(library_map) => {
                                    for library in library_map {
                                        let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                                        for path in library.1.1.downloads.artifact.as_ref().unwrap().path.split('/') {
                                            path_vector.push(path);
                                        }
                                        let path: PathBuf = common::join_directories(path_vector).unwrap();
                                        paths.push(path);
                                    }
                                },
                                None => {}
                            }

                            match get_required_natives(&version) {
                                Some(natives) => {
                                    for native in natives {
                                        let mut path_vector: Vec<&str> = Vec::from(["libraries"]);
                                        for path in native.path.split('/') {
                                            path_vector.push(path);
                                        }
                                        let path: PathBuf = common::join_directories(path_vector).unwrap();
                                        paths.push(path);
                                    }
                                },
                                None => {}
                            }
                            let path: PathBuf = common::join_directories(Vec::from(["libraries", "com", "mojang", "minecraft", &version.id, &*format!("{}.jar", version.id)])).unwrap();
                            paths.push(path);

                            let mut builder: String = String::new();
                            for path in paths {
                                if builder.is_empty() {
                                    builder.push_str(&path.into_os_string().into_string().unwrap());
                                } else {
                                    let os = std::env::consts::OS;
                                    if os.eq("windows") {
                                        builder.push_str(&format!(";{}", path.into_os_string().into_string().unwrap()));
                                    } else if os.eq("linux") || os.eq("macos") {
                                        builder.push_str(&format!(":{}", path.into_os_string().into_string().unwrap()));
                                    }
                                }
                            }

                            arg.replace("${classpath}", builder.as_str())
                        } else {
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
                            game_arguments.push(auth.selected_profile.as_ref().unwrap().name.as_ref().unwrap().to_owned());
                        }
                        if arg.contains("version_name") {
                            game_arguments.push(version.id.to_owned());
                        }
                        if arg.contains("game_directory") {
                            let game_directory = common::join_directories(Vec::from(["instances", &version.id, ".minecraft"])).unwrap().into_os_string().into_string().unwrap();
                            game_arguments.push(game_directory);
                        }
                        if arg.contains("assets_root") {
                            let assets_root = common::join_directories(Vec::from(["assets"])).unwrap().into_os_string().into_string().unwrap();
                            game_arguments.push(assets_root);
                        }
                        if arg.contains("assets_index_name") {
                            game_arguments.push(version.asset_index.id.to_owned());
                        }
                        if arg.contains("auth_uuid") {
                            game_arguments.push(auth.selected_profile.as_ref().unwrap().id.as_ref().unwrap().to_owned());
                        }
                        if arg.contains("auth_access_token") {
                            game_arguments.push(auth.access_token.as_ref().unwrap().to_owned());
                        }
                        if arg.contains("user_type") {
                            game_arguments.push(String::from("mojang"));
                        }
                        if arg.contains("version_type") {
                            game_arguments.push(version.r#type.to_owned());
                        }
                    } else {
                        game_arguments.push(arg.to_owned());
                    }
                }
            }
        }
        None => {}
    }
    let mut command = std::process::Command::new("java");
    for jvm_argument in jvm_arguments {
        command.arg(&jvm_argument);
        println!("{}", &jvm_argument);
    }
    command.arg(&version.main_class);
    println!("{}", &version.main_class);
    for argument in game_arguments {
        command.arg(&argument);
        println!("{}", &argument);
    }
    let output = command.output().expect("kekw");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    /*if ! {
        println!("kekw");
    }*/
}