//! Module for noughts and crosses game
pub mod converters;
pub mod playboard;
pub mod player_manager;

use player_manager::PlayerTrait;

/// Enum for game stage
#[derive(PartialEq)]
enum GameStage {
    /// Waiting for both players
    WaitingForPlayers,
    /// Waiting for specific player
    WaitingForPlayer(PlayerId),
    /// Both players are available - player is on move
    PlayerOnMove(PlayerId),
}

/// Enum for player id
#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum PlayerId {
    /// Player playing for circle
    Circle,
    /// Player playing for cross
    Cross,
}

impl std::fmt::Display for PlayerId {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Circle => "o",
            Self::Cross => "x"
        };

        write!(f, "{}", name)
    }
}

impl std::ops::Not for PlayerId {
    type Output = PlayerId;

    fn not(self) -> Self::Output {
        match self {
            Self::Circle => Self::Cross,
            Self::Cross => Self::Circle,
        }
    }
}

/// Run game
pub async fn run_game<T, U, V, W>(mut player_manager: T, create_playboard: V, converter: W)
where
    T: player_manager::PlayerManagerTrait<U>,
    U: playboard::Playboard + std::fmt::Display + std::marker::Sync,
    V: Fn() -> U,
    W: Fn(T::PlayerMsg) -> converters::ConversionResult<U::Position>,
{
    let mut game_stage = GameStage::WaitingForPlayers;

    let mut playboard = create_playboard();

    let mut players: std::collections::HashMap<PlayerId, T::NewPlayer<'_>> =
        std::collections::HashMap::with_capacity(2);

    loop {
        match player_manager.receive_new_message().await {
            player_manager::MsgFromPlayer::Join(new_player_data) => {
                match game_stage {
                    GameStage::WaitingForPlayers => {
                        let mut player =
                            player_manager.create_new_player(PlayerId::Circle, new_player_data);

                        player
                            .send_msg_to_player(player_manager::MsgToPlayer::WelcomePlayer)
                            .await;

                        player
                            .send_msg_to_player(player_manager::MsgToPlayer::WaitingForOtherPlayer)
                            .await;

                        players.insert(player.get_player_id(), player);

                        game_stage = GameStage::WaitingForPlayer(PlayerId::Cross);
                    }
                    GameStage::WaitingForPlayer(player_id) => {
                        let mut player =
                            player_manager.create_new_player(player_id, new_player_data);
                        player
                            .send_msg_to_player(player_manager::MsgToPlayer::WelcomePlayer)
                            .await;
                        players.insert(player.get_player_id(), player);

                        for player in players.values_mut() {
                            player
                                .send_msg_to_player(player_manager::MsgToPlayer::PlayersAreReady)
                                .await;
                            player
                                .send_msg_to_player(player_manager::MsgToPlayer::Playboard(
                                    &playboard,
                                ))
                                .await;
                        }

                        game_stage = GameStage::PlayerOnMove(!player_id);

                        players
                            .get_mut(&!player_id)
                            .unwrap()
                            .send_msg_to_player(player_manager::MsgToPlayer::YourAreOnMove)
                            .await;

                        players
                            .get_mut(&player_id)
                            .unwrap()
                            .send_msg_to_player(player_manager::MsgToPlayer::OtherPlayerIsOnMove)
                            .await;
                    }
                    _ => {
                        // TODO - close connection
                        println!("Both players are connected")
                    }
                }
            }
            player_manager::MsgFromPlayer::Msg(player_id, msg) => {
                if let GameStage::PlayerOnMove(player_on_move) = game_stage {
                    match player_id == player_on_move {
                        true => {
                            let player_ = players.get_mut(&player_on_move).unwrap();

                            match converter(msg) {
                                Ok(position) => match playboard.new_move(position, player_id) {
                                    Ok(res) => {
                                        match res {
                                            playboard::ValidMove::Continue => (),
                                            playboard::ValidMove::Draw => {
                                                for player in players.values_mut() {
                                                    player
                                                        .send_msg_to_player(
                                                            player_manager::MsgToPlayer::Playboard(
                                                                &playboard,
                                                            ),
                                                        )
                                                        .await;

                                                    player
                                                        .send_msg_to_player(
                                                            player_manager::MsgToPlayer::Draw,
                                                        )
                                                        .await;
                                                }
                                                playboard = create_playboard();
                                            }
                                            playboard::ValidMove::Win => {
                                                for player in players.values_mut() {
                                                    player
                                                        .send_msg_to_player(
                                                            player_manager::MsgToPlayer::Playboard(
                                                                &playboard,
                                                            ),
                                                        )
                                                        .await;
                                                }

                                                players
                                                    .get_mut(&player_on_move)
                                                    .unwrap()
                                                    .send_msg_to_player(
                                                        player_manager::MsgToPlayer::YouWon,
                                                    )
                                                    .await;

                                                players
                                                    .get_mut(&!player_on_move)
                                                    .unwrap()
                                                    .send_msg_to_player(
                                                        player_manager::MsgToPlayer::YouLose,
                                                    )
                                                    .await;

                                                playboard = create_playboard();
                                            }
                                        }

                                        players
                                            .get_mut(&!player_on_move)
                                            .unwrap()
                                            .send_msg_to_player(
                                                player_manager::MsgToPlayer::Playboard(&playboard),
                                            )
                                            .await;
                                        players
                                            .get_mut(&player_on_move)
                                            .unwrap()
                                            .send_msg_to_player(
                                                player_manager::MsgToPlayer::Playboard(&playboard),
                                            )
                                            .await;

                                        players
                                            .get_mut(&!player_on_move)
                                            .unwrap()
                                            .send_msg_to_player(
                                                player_manager::MsgToPlayer::YourAreOnMove,
                                            )
                                            .await;
                                        players
                                            .get_mut(&player_on_move)
                                            .unwrap()
                                            .send_msg_to_player(
                                                player_manager::MsgToPlayer::OtherPlayerIsOnMove,
                                            )
                                            .await;

                                        game_stage = GameStage::PlayerOnMove(!player_on_move);
                                    }
                                    Err(err) => match err {
                                        playboard::InvalidMove::AlreadyUsed => {
                                            player_
                                                .send_msg_to_player(
                                                    player_manager::MsgToPlayer::AlreadyTaken,
                                                )
                                                .await;
                                        }
                                        playboard::InvalidMove::InvalidRange => {
                                            player_
                                                .send_msg_to_player(
                                                    player_manager::MsgToPlayer::InvalidInput,
                                                )
                                                .await
                                        }
                                    },
                                },
                                Err(_) => {
                                    player_
                                        .send_msg_to_player(
                                            player_manager::MsgToPlayer::InvalidInput,
                                        )
                                        .await
                                }
                            };
                        }
                        false => {
                            players
                                .get_mut(&player_id)
                                .unwrap()
                                .send_msg_to_player(player_manager::MsgToPlayer::YouAreNotOnMove)
                                .await
                        }
                    }
                };
            }

            // Client disconnected
            player_manager::MsgFromPlayer::Leave(id) => {
                players.remove(&id);

                if let GameStage::PlayerOnMove(_) = game_stage {
                    playboard = create_playboard();
                }

                if players.len() == 1 {
                    let player_ = players.values_mut().next().unwrap();

                    player_
                        .send_msg_to_player(player_manager::MsgToPlayer::OtherPlayerLeave)
                        .await;

                    player_
                        .send_msg_to_player(player_manager::MsgToPlayer::WaitingForOtherPlayer)
                        .await;

                    game_stage = GameStage::WaitingForPlayer(id);
                } else {
                    game_stage = GameStage::WaitingForPlayers;
                }
            }
        };
    }
}
