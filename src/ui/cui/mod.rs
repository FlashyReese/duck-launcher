fn read_line() -> Option<u8>{
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).expect("Something went wrong!");
    match line.parse(){
        Ok(val) => Some(val),
        Err(e) => {
            println!("Invalid Input! Try Again");
            read_line()
        }
    }
}