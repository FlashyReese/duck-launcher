use std::io;

mod common;

#[tokio::main]
async fn main() {
    let mut email = String::new();
    let mut password = String::new();

    println!("Email: ");
    io::stdin().read_line(&mut email).expect("Something went wrong!");
    println!("Password: ");
    io::stdin().read_line(&mut password).expect("Something went wrong!");

    let response = common::auth::yggdrasil::authenticate(&email.trim(), &password.trim(), "").await;

    let value = response.unwrap();

    match value{
        Some(authentication_response) => {
            if authentication_response.error == None {
                match common::minecraft::get_version("1.16.5").await {
                    Ok(option_version) => {
                        match option_version {
                            Some(version) => {
                                common::minecraft::download_assets(&version).await;
                                common::minecraft::download_libraries(&version).await;
                                common::minecraft::download_natives(&version).await;
                                common::minecraft::download_client(&version).await;
                                common::minecraft::launch_client(&authentication_response ,&version);
                            }
                            None => {}
                        }
                    }
                    Err(e) => panic!("{}", e)
                }
            }else {
                println!("Login Error: {}", authentication_response.error_message.unwrap())
            }
        }
        None => println!("None")
    }
}