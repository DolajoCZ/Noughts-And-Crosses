mod field;
mod msgs;
mod player;

// ---- Common ----

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum PlayerName {
    Circle,
    Cross,
}

impl std::fmt::Display for PlayerName {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Circle => "o",
            Self::Cross => "x"
        };
        
        write!(f, "{}", name)
    }
}





pub enum Msg {
    NewConnection(std::net::SocketAddr, tokio::net::TcpStream),
    FromClient(PlayerName, String),
    Disconnect(PlayerName),
}

// ---- Manage incoming connections ----

async fn connection_listener(
    listener: tokio::net::TcpListener,
    tx: tokio::sync::mpsc::Sender<Msg>,
) {
    loop {
        let (stream, address) = listener.accept().await.unwrap();
        tx.send(Msg::NewConnection(address, stream)).await;
    }
}

// ---- Messages ----

async fn send_new_game_round(
    player_on_move: &player::Player,
    player_waiting: &player::Player,
    field: &field::Field,
) {
    msgs::send_field(player_on_move, field).await;
    msgs::send_field(player_waiting, field).await;

    msgs::send_you_move(player_on_move).await;
    msgs::send_other_player_is_on_move(player_waiting).await;
}

#[derive(PartialEq)]
enum GameStage {
    WaitingForPlayers,
    WaitingForPlayer(PlayerName),
    PlayerOnMove,
}

async fn game(
    tx_game: tokio::sync::mpsc::Sender<Msg>,
    mut rx_game: tokio::sync::mpsc::Receiver<Msg>,
) {
    let mut field = field::Field::new();
    let mut game_stage = GameStage::WaitingForPlayers;

    let mut players: std::collections::HashMap<PlayerName, player::Player> =
        std::collections::HashMap::with_capacity(2);

    let mut player_on_move = PlayerName::Circle;
    let mut player_waiting = PlayerName::Cross;

    loop {
        match rx_game.recv().await.unwrap() {
            // Connected new player
            Msg::NewConnection(_, stream) => {
                match game_stage {
                    GameStage::WaitingForPlayers => {
                        let player =
                            player::Player::new(stream, PlayerName::Circle, tx_game.clone());

                        msgs::send_welcome_player(&player).await;
                        msgs::send_waiting_for_another_player(&player).await;
                        players.insert(player.get_name(), player);

                        game_stage = GameStage::WaitingForPlayer(PlayerName::Cross);
                    }
                    GameStage::WaitingForPlayer(player_name) => {
                        let player = player::Player::new(stream, player_name, tx_game.clone());
                        msgs::send_welcome_player(&player).await;
                        players.insert(player.get_name(), player);

                        for player in players.values() {
                            msgs::send_both_players_ready(player).await;
                            msgs::send_field(player, &field).await;
                        }

                        game_stage = GameStage::PlayerOnMove;

                        msgs::send_you_move(players.get(&player_on_move).unwrap()).await;
                        msgs::send_other_player_is_on_move(players.get(&player_waiting).unwrap())
                            .await;
                    }
                    _ => {
                        // TODO - close connection
                        println!("Both players are connected")
                    }
                }
            }
            // Message from client
            Msg::FromClient(player_name, msg) => match player_name == player_on_move {
                true => {
                    let player_ = players.get(&player_on_move).unwrap();

                    match field.new_move(&msg, player_name) {
                        Ok(res) => {
                            match res {
                                field::ValidMove::Continue => (),
                                field::ValidMove::Draw => {
                                    for player in players.values() {
                                        msgs::send_field(player, &field).await;
                                        msgs::send_draw(player).await
                                    }
                                    field = field::Field::new();
                                }
                                field::ValidMove::Win => {
                                    for player in players.values() {
                                        msgs::send_field(player, &field).await;
                                    }

                                    msgs::send_win(players.get(&player_on_move).unwrap()).await;
                                    msgs::send_lose(players.get(&player_waiting).unwrap()).await;

                                    field = field::Field::new();
                                }
                            }

                            (player_on_move, player_waiting) = (player_waiting, player_on_move);

                            send_new_game_round(
                                players.get(&player_on_move).unwrap(),
                                players.get(&player_waiting).unwrap(),
                                &field,
                            )
                            .await;
                        }
                        Err(err) => match err {
                            field::InvalidMove::AlreadyUsed => {
                                msgs::send_already_taken(player_).await
                            }
                            _ => msgs::send_invalid_input(player_).await,
                        },
                    };
                }
                false => msgs::send_you_are_not_on_move(players.get(&player_name).unwrap()).await,
            },

            // Client disconnected
            Msg::Disconnect(id) => {
                players.remove(&id);

                if game_stage == GameStage::PlayerOnMove {
                    field = field::Field::new();
                }

                if players.len() == 1 {
                    let player_ = players.values().next().unwrap();

                    if player_.get_name() == player_on_move {
                        player_.send_msg_to_player("\n".to_owned()).await
                    }

                    msgs::send_players_leave_game(player_).await;
                    msgs::send_waiting_for_another_player(player_).await;
                    game_stage = GameStage::WaitingForPlayer(id);
                } else {
                    game_stage = GameStage::WaitingForPlayers;
                }
            }
        };
    }
}

pub async fn run(server_address: &str) {
    let listener = match tokio::net::TcpListener::bind(server_address).await {
        Ok(x) => x,
        Err(err) => {
            println!("Fail to create new TCP listener: {:?}", err);
            std::process::exit(2);
        }
    };

    let (tx, rx) = tokio::sync::mpsc::channel(10);

    tokio::join!(connection_listener(listener, tx.clone()), game(tx, rx));
}
