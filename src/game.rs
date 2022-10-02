mod field;
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

// ---- Messages - info ----

async fn send_welcome_player(player: &player::Player) {
    player
        .send_msg_to_player(format!("Welcome player {:?}", player.get_name()))
        .await;
}

async fn send_waiting_for_another_player(player: &player::Player) {
    player
        .send_msg_to_player("We are waiting fro another player".to_string())
        .await;
}

async fn send_both_players_ready(player: &player::Player) {
    player
        .send_msg_to_player("Both players are ready".to_string())
        .await;
}

async fn send_players_leave_game(player: &player::Player) {
    player
        .send_msg_to_player("Other player leave game.\nCongratulation you win!".to_string())
        .await;
}

async fn send_you_move(player: &player::Player) {
    player
        .send_msg_to_player("Now you are on move.".to_string())
        .await;
}

async fn send_other_player_is_on_move(player: &player::Player) {
    player
        .send_msg_to_player("Now is other player on move.".to_string())
        .await;
}

async fn send_draw(player: &player::Player) {
    player.send_msg_to_player("Nobody win.".to_string()).await;
}

async fn send_win(player: &player::Player) {
    player
        .send_msg_to_player("Congratulation, you win.".to_string())
        .await;
}

async fn send_lose(player: &player::Player) {
    player
        .send_msg_to_player("Unfortunately you lose.".to_string())
        .await;
}

async fn send_field(player: &player::Player, field: &field::Field) {
    player
        .send_msg_to_player(format!(
            "-------------------\nCurrent game field\n{}",
            field
        ))
        .await;
}

// ---- Messages - error ----

async fn send_you_are_not_on_move(player: &player::Player) {
    player
        .send_msg_to_player("You are not on move. Please wait till other player move".to_string())
        .await;
}

async fn send_invalid_input(player: &player::Player) {
    player
        .send_msg_to_player("You pass invalid input. Please repeat your input".to_string())
        .await;
}

async fn send_already_taken(player: &player::Player) {
    player
        .send_msg_to_player("Required field is already taken. Please repeat your input".to_string())
        .await;
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
    // let mut players_iter = [Symbols::Circle, Symbols::Cross].iter().cycle();

    let mut players: std::collections::HashMap<field::Symbols, player::Player> =
        std::collections::HashMap::with_capacity(2);

    let mut player_on_move = field::Symbols::Circle;
    let mut player_waiting = field::Symbols::Cross;

    loop {
        match rx_game.recv().await.unwrap() {
            // Connected new player
            Msg::NewConnection(_, mut stream) => {
                match game_stage {
                    GameStage::WaitingForPlayers => {
                        let player =
                            player::Player::new(stream, field::Symbols::Circle, tx_game.clone());

                        send_welcome_player(&player).await;
                        send_waiting_for_another_player(&player).await;
                        players.insert(player.get_name(), player);

                        game_stage = GameStage::WaitingForPlayer(field::Symbols::Cross);
                    }
                    GameStage::WaitingForPlayer(x) => {
                        let player = player::Player::new(stream, x, tx_game.clone());
                        send_welcome_player(&player).await;
                        players.insert(player.get_name(), player);

                        for player in players.values() {
                            send_both_players_ready(player).await;
                            send_field(player, &field).await;
                        }

                        game_stage = GameStage::PlayerOnMove;

                        send_you_move(players.get(&player_on_move).unwrap()).await;
                        send_other_player_is_on_move(players.get(&player_waiting).unwrap()).await;
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
                                    send_field(player, &field).await;
                                }

                                (player_on_move, player_waiting) = (player_waiting, player_on_move);
                                send_you_move(players.get(&player_on_move).unwrap()).await;
                                send_other_player_is_on_move(players.get(&player_waiting).unwrap())
                                    .await;
                            }
                            field::ValidMove::Draw => {
                                for player in players.values() {
                                    send_field(player, &field).await;
                                    send_draw(player).await
                                }
                                field = field::Field::new();
                            }
                            field::ValidMove::Win => {
                                for player in players.values() {
                                    send_field(player, &field).await;
                                }

                                send_win(players.get(&player_on_move).unwrap()).await;
                                send_lose(players.get(&player_waiting).unwrap()).await;
                                (player_on_move, player_waiting) = (player_waiting, player_on_move);
                                field = field::Field::new();
                            }
                        },
                        Err(err) => match err {
                            field::InvalidMove::AlreadyUsed => send_already_taken(player_).await,
                            _ => send_invalid_input(player_).await,
                        },
                    };
                }
                false => send_you_are_not_on_move(players.get(&id).unwrap()).await,
            },

            // Client disconnected
            Msg::Disconnect(id) => {
                players.remove(&id);

                if players.len() == 1 {
                    send_players_leave_game(players.values().next().unwrap()).await;
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
