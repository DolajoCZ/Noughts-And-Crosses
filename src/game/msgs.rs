use super::field;
use super::player;

// ---- Messages - info ----
pub async fn send_welcome_player(player: &player::Player) {
    player
        .send_msg_to_player(format!("Welcome player {:?}\n", player.get_name()))
        .await;
}

pub async fn send_waiting_for_another_player(player: &player::Player) {
    player
        .send_msg_to_player("We are waiting fro another player\n".to_string())
        .await;
}

pub async fn send_both_players_ready(player: &player::Player) {
    player
        .send_msg_to_player("Both players are ready\n".to_string())
        .await;
}

pub async fn send_players_leave_game(player: &player::Player) {
    player
        .send_msg_to_player("Other player leave game.\nCongratulation you win!\n".to_string())
        .await;
}

pub async fn send_you_move(player: &player::Player) {
    player
        .send_msg_to_player("Now you are on move: ".to_string())
        .await;
}

pub async fn send_other_player_is_on_move(player: &player::Player) {
    player
        .send_msg_to_player("Now is other player on move\n".to_string())
        .await;
}

pub async fn send_draw(player: &player::Player) {
    player.send_msg_to_player("Nobody win\n".to_string()).await;
}

pub async fn send_win(player: &player::Player) {
    player
        .send_msg_to_player("Congratulation, you win.\n".to_string())
        .await;
}

pub async fn send_lose(player: &player::Player) {
    player
        .send_msg_to_player("Unfortunately you lose.\n".to_string())
        .await;
}

pub async fn send_field(player: &player::Player, field: &field::Field) {
    player
        .send_msg_to_player(format!(
            "------------------\nCurrent game field\n\n{}\n",
            field
        ))
        .await;
}

// ---- Messages - error ----

pub async fn send_you_are_not_on_move(player: &player::Player) {
    player
        .send_msg_to_player("You are not on move. Please wait till other player move\n".to_string())
        .await;
}

pub async fn send_invalid_input(player: &player::Player) {
    player
        .send_msg_to_player("You pass invalid input. Please repeat your input: ".to_string())
        .await;
}

pub async fn send_already_taken(player: &player::Player) {
    player
        .send_msg_to_player(
            "Required field is already taken. Please repeat your input: ".to_string(),
        )
        .await;
}
