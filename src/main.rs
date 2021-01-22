use std::io;
use crate::minecraft::version::Version;
use crate::minecraft::{Instance, InstanceType, InstanceFlavor};
use crate::minecraft::dependency::LibrariesMetadata;

mod common;
mod minecraft;
mod ui;

#[tokio::main]
async fn main() {
    let mut email = String::new();
    let mut password = String::new();
    let mut version = String::new();

    println!("Email: ");
    io::stdin().read_line(&mut email).expect("Something went wrong!");
    println!("Password: ");
    io::stdin().read_line(&mut password).expect("Something went wrong!");
    println!("Version: ");
    io::stdin().read_line(&mut version).expect("Something went wrong!");

    let response = minecraft::yggdrasil::authenticate(&email.trim(), &password.trim(), "").await;

    let value = response.unwrap();

    match value {
        Some(authentication_response) => {
            if authentication_response.error == None {
                match Version::get_version(&version.trim()).await {
                    Ok(option_version) => {
                        if let Some(version) = option_version {
                            let libs_meta = LibrariesMetadata::new().push_mc_version(&version).await;
                            libs_meta.save();
                            println!("Fetching Assets");
                            version.verify_assets().await;
                            println!("Fetching Libraries");
                            version.verify_libraries().await;
                            println!("Fetching Natives");
                            version.verify_natives().await;
                            println!("Fetching Client");
                            version.verify_client().await;
                            minecraft::launch_client(&authentication_response, &version);
                        }
                    }
                    Err(e) => panic!("{}", e)
                }
            } else {
                println!("Login Error: {}", authentication_response.error_message.unwrap())
            }
        }
        None => println!("None")
    }
}