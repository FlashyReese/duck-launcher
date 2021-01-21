use std::io;

mod common;
mod minecraft;

#[tokio::main]
async fn main() {
    let mut email = String::new();
    let mut password = String::new();

    println!("Email: ");
    io::stdin().read_line(&mut email).expect("Something went wrong!");
    println!("Password: ");
    io::stdin().read_line(&mut password).expect("Something went wrong!");

    let response = minecraft::yggdrasil::authenticate(&email.trim(), &password.trim(), "").await;

    let value = response.unwrap();

    match value {
        Some(authentication_response) => {
            if authentication_response.error == None {
                match minecraft::version::get_version("1.16.5").await {
                    Ok(option_version) => {
                        match option_version {
                            Some(version) => {
                                println!("Fetching Assets");
                                minecraft::asset::verify_assets(&version).await;
                                println!("Fetching Libraries");
                                minecraft::dependency::verify_libraries(&version).await;
                                println!("Fetching Natives");
                                minecraft::dependency::verify_natives(&version).await;
                                println!("Fetching Client");
                                minecraft::download_client(&version).await;
                                minecraft::launch_client(&authentication_response, &version);
                            }
                            None => {}
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