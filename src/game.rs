mod field;
mod msgs;
mod player;

// ---- Common ----

pub enum Msg {
    NewConnection(std::net::SocketAddr, tokio::net::TcpStream),
    FromClient(field::Symbols, String),
    Disconnect(field::Symbols),
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

#[derive(PartialEq)]
enum GameStage {
    WaitingForPlayers,
    WaitingForPlayer(field::Symbols),
    PlayerOnMove,
}

async fn game(
    tx_game: tokio::sync::mpsc::Sender<Msg>,
    mut rx_game: tokio::sync::mpsc::Receiver<Msg>,
) {
    let mut field = field::Field::new();
    let mut game_stage = GameStage::WaitingForPlayers;

    let mut players: std::collections::HashMap<field::Symbols, player::Player> =
        std::collections::HashMap::with_capacity(2);

    let mut player_on_move = field::Symbols::Circle;
    let mut player_waiting = field::Symbols::Cross;

    loop {
        match rx_game.recv().await.unwrap() {
            // Connected new player
            Msg::NewConnection(_, stream) => {
                match game_stage {
                    GameStage::WaitingForPlayers => {
                        let player =
                            player::Player::new(stream, field::Symbols::Circle, tx_game.clone());

                        msgs::send_welcome_player(&player).await;
                        msgs::send_waiting_for_another_player(&player).await;
                        players.insert(player.get_name(), player);

                        game_stage = GameStage::WaitingForPlayer(field::Symbols::Cross);
                    }
                    GameStage::WaitingForPlayer(x) => {
                        let player = player::Player::new(stream, x, tx_game.clone());
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
            Msg::FromClient(id, msg) => match id == player_on_move {
                true => {
                    let player_ = players.get(&player_on_move).unwrap();

                    match field.new_move(&msg, id) {
                        Ok(res) => match res {
                            field::ValidMove::Continue => {
                                for player in players.values() {
                                    msgs::send_field(player, &field).await;
                                }

                                (player_on_move, player_waiting) = (player_waiting, player_on_move);
                                msgs::send_you_move(players.get(&player_on_move).unwrap()).await;
                                msgs::send_other_player_is_on_move(
                                    players.get(&player_waiting).unwrap(),
                                )
                                .await;
                            }
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
                                (player_on_move, player_waiting) = (player_waiting, player_on_move);
                                field = field::Field::new();
                            }
                        },
                        Err(err) => match err {
                            field::InvalidMove::AlreadyUsed => {
                                msgs::send_already_taken(player_).await
                            }
                            _ => msgs::send_invalid_input(player_).await,
                        },
                    };
                }
                false => msgs::send_you_are_not_on_move(players.get(&id).unwrap()).await,
            },

            // Client disconnected
            Msg::Disconnect(id) => {
                players.remove(&id);

                if players.len() == 1 {
                    msgs::send_players_leave_game(players.values().next().unwrap()).await;
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
