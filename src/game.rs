pub mod playboard;
pub mod player_manager;

use player_manager::PlayerTrait;

// ---- Common ----

#[derive(PartialEq)]
enum GameStage {
    WaitingForPlayers,
    WaitingForPlayer(PlayerName),
    PlayerOnMove,
}

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

pub async fn run_game<T, R>(mut player_manager: T, mut playboard: R)
where
    T: player_manager::PlayerMangerTrait,
    R: playboard::Playboard + std::fmt::Display,
{
    // let mut field = field::Field::new();
    let mut game_stage = GameStage::WaitingForPlayers;

    let mut players: std::collections::HashMap<PlayerName, T::Player> =
        std::collections::HashMap::with_capacity(2);

    let mut player_on_move = PlayerName::Circle;
    let mut player_waiting = PlayerName::Cross;

    loop {
        match player_manager.receive_new_message().await {
            player_manager::Msg::NewConnection(new_player_data) => {
                match game_stage {
                    GameStage::WaitingForPlayers => {
                        let player =
                            player_manager.create_new_player(PlayerName::Circle, new_player_data);
                        // player::Player::new(stream, PlayerName::Circle, tx_game.clone());

                        player_manager::msgs::send_welcome_player(&player).await;
                        player_manager::msgs::send_waiting_for_another_player(&player).await;
                        players.insert(player.get_name(), player);

                        game_stage = GameStage::WaitingForPlayer(PlayerName::Cross);
                    }
                    GameStage::WaitingForPlayer(player_name) => {
                        let player = player_manager.create_new_player(player_name, new_player_data);
                        player_manager::msgs::send_welcome_player(&player).await;
                        players.insert(player.get_name(), player);

                        for player in players.values() {
                            player_manager::msgs::send_both_players_ready(player).await;
                            player_manager::msgs::send_field(player, &playboard).await;
                        }

                        game_stage = GameStage::PlayerOnMove;

                        player_manager::msgs::send_you_move(players.get(&player_on_move).unwrap())
                            .await;
                        player_manager::msgs::send_other_player_is_on_move(
                            players.get(&player_waiting).unwrap(),
                        )
                        .await;
                    }
                    _ => {
                        // TODO - close connection
                        println!("Both players are connected")
                    }
                }
            }
            player_manager::Msg::FromClient(player_name, msg) => {
                match player_name == player_on_move {
                    true => {
                        let player_ = players.get(&player_on_move).unwrap();

                        match playboard.new_move(&msg, player_name) {
                            Ok(res) => {
                                match res {
                                    playboard::ValidMove::Continue => (),
                                    playboard::ValidMove::Draw => {
                                        for player in players.values() {
                                            player_manager::msgs::send_field(player, &playboard)
                                                .await;
                                            player_manager::msgs::send_draw(player).await
                                        }
                                        playboard.reset();
                                    }
                                    playboard::ValidMove::Win => {
                                        for player in players.values() {
                                            player_manager::msgs::send_field(player, &playboard)
                                                .await;
                                        }

                                        player_manager::msgs::send_win(
                                            players.get(&player_on_move).unwrap(),
                                        )
                                        .await;
                                        player_manager::msgs::send_lose(
                                            players.get(&player_waiting).unwrap(),
                                        )
                                        .await;

                                        playboard.reset();
                                    }
                                }

                                (player_on_move, player_waiting) = (player_waiting, player_on_move);

                                player_manager::msgs::send_new_game_round(
                                    players.get(&player_on_move).unwrap(),
                                    players.get(&player_waiting).unwrap(),
                                    &playboard,
                                )
                                .await;
                            }
                            Err(err) => match err {
                                playboard::InvalidMove::AlreadyUsed => {
                                    player_manager::msgs::send_already_taken(player_).await
                                }
                                _ => player_manager::msgs::send_invalid_input(player_).await,
                            },
                        };
                    }
                    false => {
                        player_manager::msgs::send_you_are_not_on_move(
                            players.get(&player_name).unwrap(),
                        )
                        .await
                    }
                }
            }

            // Client disconnected
            player_manager::Msg::Disconnect(id) => {
                players.remove(&id);

                if game_stage == GameStage::PlayerOnMove {
                    playboard.reset();
                }

                if players.len() == 1 {
                    let player_ = players.values().next().unwrap();

                    if player_.get_name() == player_on_move {
                        player_.send_msg_to_player("\r\n".to_owned()).await
                    }

                    player_manager::msgs::send_players_leave_game(player_).await;
                    player_manager::msgs::send_waiting_for_another_player(player_).await;
                    game_stage = GameStage::WaitingForPlayer(id);
                } else {
                    game_stage = GameStage::WaitingForPlayers;
                }
            }
        };
    }
}
