// #![warn(clippy::missing_docs_in_private_items)]
//! This is crate for playing noughts and crosses game
mod game;

///Get user arguments from cmd
fn get_args() -> clap::ArgMatches {
    clap::Command::new("Noughts and crosses")
        .args(vec![
            clap::Arg::new("port")
                .short('p')
                .long("port")
                .required(false)
                .help("Custom required port")
                .value_parser(clap::value_parser!(u16)),
            clap::Arg::new("playboard_size")
                .short('s')
                .long("size")
                .required(false)
                .help("Size of playboard edge")
                .value_parser(3..=10)
                .default_value("3"),
        ])
        .get_matches()
}

/// Set logger
fn set_logger() -> Result<(), log::SetLoggerError> {
    simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )])
}

#[tokio::main]
async fn main() {
    let args = get_args();

    // Set logger
    match set_logger() {
        Ok(()) => (),
        Err(_) => std::process::exit(1),
    }

    log::info!("Starting game");

    let ip_address = match game::player_manager::tcp::select_network() {
        Ok(addr) => addr,
        Err(()) => std::process::exit(1),
    };

    let r_player_manager = match args.get_one::<u16>("port") {
        Some(port) => {
            let addr = std::net::SocketAddr::new(ip_address, port.to_owned());
            game::player_manager::tcp::PlayerManager::from_socket_address(addr).await
        }
        None => game::player_manager::tcp::PlayerManager::from_ip(ip_address).await,
    };

    let player_manager = match r_player_manager {
        Ok(x) => x,
        Err(err) => {
            log::error!(
                "Unable to create TCP player manager due to following error: {}",
                err
            );
            std::process::exit(2)
        }
    };

    let playboard_size = args.get_one::<i64>("playboard_size").unwrap().to_owned() as usize;
    let playboard_builder = move || game::playboard::pb_n_n::Playboard::new(playboard_size);

    game::run_game(
        player_manager,
        playboard_builder,
        game::converters::pm_tcp_msg_to_x_y,
        game::converters::pb_n_n_to_string,
    )
    .await;
}
