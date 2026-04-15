pub fn register() -> bool {
    println!("Enter username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).expect("Failed to read line");

    println!("Enter email:");
    let mut email = String::new();
    std::io::stdin().read_line(&mut email).expect("Failed to read line");

    println!("Enter password:");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).expect("Failed to read line");

    println!("User '{}' registered successfully!", username.trim());
    true
}

pub fn login() -> bool{
    println!("Enter username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).expect("Failed to read line");

    println!("Enter password:");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).expect("Failed to read line");

    println!("User '{}' logged in successfully!", username.trim());
    true
}