mod game;

use std::io::Write;

fn get_available_interfaces() -> Result<Vec<(String, std::net::IpAddr)>, ()> {
    let network_interfaces = local_ip_address::list_afinet_netifas().map_err(|err| {
        println!("Unable to load network interfaces: {}", err);
        ()
    })?;

    println!("Available interfaces are:");
    for (index, (name, addr)) in network_interfaces.iter().enumerate() {
        println!("Id: {} - Name: {} - Ip address: {}", index, name, addr);
    }

    Ok(network_interfaces)
}

fn get_user_input() -> Result<usize, ()> {
    print!("Please select interface Id: ");
    std::io::stdout().flush().map_err(|err| {
        println!("Unable to read user input: {}", err);
        ()
    })?;

    // Read user input
    let mut user_input = String::new();

    let length = std::io::stdin().read_line(&mut user_input).map_err(|err| {
        println!("Unable to read user input: {}", err);
        ()
    })?;

    let user_input = &user_input[..length];

    let user_input = user_input
        .strip_suffix("\r\n")
        .unwrap_or(user_input.strip_suffix("\n").unwrap_or(user_input));

    user_input.parse().map_err(|_| {
        println!(
            "Unable to convert \"{}\" to integer in range [0 - {}]",
            &user_input,
            usize::MAX
        );
        ()
    })
}

fn select_network() -> Result<std::net::IpAddr, ()> {
    let interfaces = get_available_interfaces()?;
    let index = get_user_input()?;

    match interfaces.get(index) {
        Some(x) => Ok(x.1),
        None => Err(()),
    }
}

#[tokio::main]
async fn main() {
    let port = 8000;
    let address = match select_network() {
        Ok(addr) => addr,
        Err(()) => std::process::exit(1),
    };

    let addr = std::net::SocketAddr::new(address, port);

    println!("Game server running on address: {}:{}", address, port);
    let player_manager = game::player_manager::tcp::PlayerManager::new(addr).await;

    game::run_game(player_manager, game::playboard::bp_3_3::Playboard::new).await;
}
