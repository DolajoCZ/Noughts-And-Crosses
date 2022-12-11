pub mod playboard;
pub mod player_manager;

use player_manager::PlayerTrait;

// ---- Common ----

#[derive(PartialEq)]
enum GameStage {
    WaitingForPlayers,
    WaitingForPlayer(PlayerId),
    PlayerOnMove,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum PlayerId {
    Circle,
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

pub async fn run_game<T, F, R>(mut player_manager: T, create_playboard: F)
where
    T: player_manager::PlayerMangerTrait<R>,
    F: Fn() -> R,
    R: playboard::Playboard + std::fmt::Display + std::marker::Sync,
{
    let mut game_stage = GameStage::WaitingForPlayers;

    let mut playboard = create_playboard();

    let mut players: std::collections::HashMap<PlayerId, T::NewPlayer<'_>> =
        std::collections::HashMap::with_capacity(2);

    let mut player_on_move = PlayerId::Circle;
    let mut player_waiting = PlayerId::Cross;

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

                        game_stage = GameStage::PlayerOnMove;

                        players
                            .get_mut(&player_on_move)
                            .unwrap()
                            .send_msg_to_player(player_manager::MsgToPlayer::YourAreOnMove)
                            .await;

                        players
                            .get_mut(&player_waiting)
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
                match player_id == player_on_move {
                    true => {
                        let player_ = players.get_mut(&player_on_move).unwrap();

                        match playboard.new_move(&msg, player_id) {
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
                                            .send_msg_to_player(player_manager::MsgToPlayer::YouWon)
                                            .await;

                                        players
                                            .get_mut(&player_waiting)
                                            .unwrap()
                                            .send_msg_to_player(
                                                player_manager::MsgToPlayer::YouLose,
                                            )
                                            .await;

                                        playboard = create_playboard();
                                    }
                                }

                                (player_on_move, player_waiting) = (player_waiting, player_on_move);

                                players
                                    .get_mut(&player_on_move)
                                    .unwrap()
                                    .send_msg_to_player(player_manager::MsgToPlayer::Playboard(
                                        &playboard,
                                    ))
                                    .await;
                                players
                                    .get_mut(&player_waiting)
                                    .unwrap()
                                    .send_msg_to_player(player_manager::MsgToPlayer::Playboard(
                                        &playboard,
                                    ))
                                    .await;

                                players
                                    .get_mut(&player_on_move)
                                    .unwrap()
                                    .send_msg_to_player(player_manager::MsgToPlayer::YourAreOnMove)
                                    .await;
                                players
                                    .get_mut(&player_waiting)
                                    .unwrap()
                                    .send_msg_to_player(
                                        player_manager::MsgToPlayer::OtherPlayerIsOnMove,
                                    )
                                    .await;
                            }
                            Err(err) => match err {
                                playboard::InvalidMove::AlreadyUsed => {
                                    player_
                                        .send_msg_to_player(
                                            player_manager::MsgToPlayer::AlreadyTaken,
                                        )
                                        .await;
                                }
                                _ => {
                                    player_
                                        .send_msg_to_player(
                                            player_manager::MsgToPlayer::InvalidInput,
                                        )
                                        .await
                                }
                            },
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
            }

            // Client disconnected
            player_manager::MsgFromPlayer::Leave(id) => {
                players.remove(&id);

                if game_stage == GameStage::PlayerOnMove {
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
