pub async fn send_new_game_round<T, R>(player_on_move: &T, player_waiting: &T, field: &R)
where
    T: super::PlayerTrait,
    R: std::fmt::Display,
{
    send_field(player_on_move, field).await;
    send_field(player_waiting, field).await;

    send_you_move(player_on_move).await;
    send_other_player_is_on_move(player_waiting).await;
}

fn add_player_name_prefix<T>(msg: &str, player: &T) -> String
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    format!("[{}] {}", player.get_name(), msg)
}

// ---- Messages - info ----
pub async fn send_welcome_player<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix("Welcome player\r\n", player))
        .await;
}

pub async fn send_waiting_for_another_player<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix(
            "We are waiting for another player\r\n",
            player,
        ))
        .await;
}

pub async fn send_both_players_ready<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix("Both players are ready\r\n", player))
        .await;
}

pub async fn send_players_leave_game<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix(
            "Other player leave game. Congratulation you win!\r\n",
            player,
        ))
        .await;
}

pub async fn send_you_move<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix("Now you are on move: ", player))
        .await;
}

pub async fn send_other_player_is_on_move<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix(
            "Now is other player on move\r\n",
            player,
        ))
        .await;
}

pub async fn send_draw<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix("Nobody win\r\n", player))
        .await;
}

pub async fn send_win<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix(
            "Congratulation, you win.\r\n",
            player,
        ))
        .await;
}

pub async fn send_lose<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix(
            "Unfortunately you lose.\r\n",
            player,
        ))
        .await;
}

pub async fn send_field<T, R>(player: &T, field: &R)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
    R: std::fmt::Display,
{
    player
        .send_msg_to_player(format!(
            "------------------\r\nCurrent game field\r\n\r\n{}\r\n",
            field
        ))
        .await;
}

// ---- Messages - error ----
pub async fn send_you_are_not_on_move<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix(
            "You are not on move. Please wait till other player move\r\n",
            player,
        ))
        .await;
}

pub async fn send_invalid_input<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(add_player_name_prefix(
            "You pass invalid input. Please repeat your input: ",
            player,
        ))
        .await;
}

pub async fn send_already_taken<T>(player: &T)
where
    T: super::PlayerTrait,
    // <T as super::PlayerTrait>::T: std::fmt::Display,
{
    player
        .send_msg_to_player(
            add_player_name_prefix(
                "Required field is already taken. Please repeat your input: ",
                player,
            )
            .to_string(),
        )
        .await;
}
