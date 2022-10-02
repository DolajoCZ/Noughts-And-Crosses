// ---- Messages - info ----
pub async fn send_welcome_player(player: &super::player::Player) {
    player
        .send_msg_to_player(format!("Welcome player {:?}\n", player.get_name()))
        .await;
}

pub async fn send_waiting_for_another_player(player: &super::player::Player) {
    player
        .send_msg_to_player("We are waiting fro another player\n".to_string())
        .await;
}

pub async fn send_both_players_ready(player: &super::player::Player) {
    player
        .send_msg_to_player("Both players are ready\n".to_string())
        .await;
}

pub async fn send_players_leave_game(player: &super::player::Player) {
    player
        .send_msg_to_player("Other player leave game.\nCongratulation you win!\n".to_string())
        .await;
}

pub async fn send_you_move(player: &super::player::Player) {
    player
        .send_msg_to_player("Now you are on move: ".to_string())
        .await;
}

pub async fn send_other_player_is_on_move(player: &super::player::Player) {
    player
        .send_msg_to_player("Now is other player on move\n".to_string())
        .await;
}

pub async fn send_draw(player: &super::player::Player) {
    player.send_msg_to_player("Nobody win\n".to_string()).await;
}

pub async fn send_win(player: &super::player::Player) {
    player
        .send_msg_to_player("Congratulation, you win.\n".to_string())
        .await;
}

pub async fn send_lose(player: &super::player::Player) {
    player
        .send_msg_to_player("Unfortunately you lose.\n".to_string())
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
        .send_msg_to_player("You are not on move. Please wait till other player move\n".to_string())
        .await;
}

pub async fn send_invalid_input(player: &super::player::Player) {
    player
        .send_msg_to_player("You pass invalid input. Please repeat your input: ".to_string())
        .await;
}

pub async fn send_already_taken(player: &super::player::Player) {
    player
        .send_msg_to_player(
            "Required field is already taken. Please repeat your input: ".to_string(),
        )
        .await;
}
