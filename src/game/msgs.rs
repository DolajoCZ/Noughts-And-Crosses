fn add_player_name_prefix(msg: &str, player: &super::player::Player) -> String {
    format!("[{}] {}", player.get_name(), msg)
}

// ---- Messages - info ----
pub async fn send_welcome_player(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix("Welcome player\n", player))
        .await;
}

pub async fn send_waiting_for_another_player(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix(
            "We are waiting for another player\n",
            player,
        ))
        .await;
}

pub async fn send_both_players_ready(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix("Both players are ready\n", player))
        .await;
}

pub async fn send_players_leave_game(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix(
            "Other player leave game. Congratulation you win!\n",
            player,
        ))
        .await;
}

pub async fn send_you_move(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix("Now you are on move: ", player))
        .await;
}

pub async fn send_other_player_is_on_move(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix(
            "Now is other player on move\n",
            player,
        ))
        .await;
}

pub async fn send_draw(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix("Nobody win\n", player))
        .await;
}

pub async fn send_win(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix("Congratulation, you win.\n", player))
        .await;
}

pub async fn send_lose(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix("Unfortunately you lose.\n", player))
        .await;
}

pub async fn send_field(player: &super::player::Player, field: &super::field::Field) {
    player
        .send_msg_to_player(format!(
            "------------------\nCurrent game field\n\n{}\n",
            field
        ))
        .await;
}

// ---- Messages - error ----
pub async fn send_you_are_not_on_move(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix(
            "You are not on move. Please wait till other player move\n",
            player,
        ))
        .await;
}

pub async fn send_invalid_input(player: &super::player::Player) {
    player
        .send_msg_to_player(add_player_name_prefix(
            "You pass invalid input. Please repeat your input: ",
            player,
        ))
        .await;
}

pub async fn send_already_taken(player: &super::player::Player) {
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
