use std::io;

mod common;

fn main() {
    let mut email = String::new();
    let mut password = String::new();

    println!("Email: ");
    io::stdin().read_line(&mut email).expect("Something went wrong!");
    println!("Password: ");
    io::stdin().read_line(&mut password).expect("Something went wrong!");

    let response = common::yggdrasil::authenticate(&email.trim(), &password.trim(), "");

    let value = response.unwrap();

    match value{
        Some(value) => {
            if value.error == None {
                if value.access_token != None {
                    println!("Hey there, {}", value.selected_profile.unwrap().name.unwrap());
                }
            }else {
                println!("Login Error: {}", value.error_message.unwrap())
            }
        },
        None => println!("None")
    }
}