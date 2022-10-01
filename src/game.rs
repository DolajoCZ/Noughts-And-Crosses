use crate::field;

use crate::player::Player;

enum GameStatus {
    WaitingForPlayers,
    Playing(u64, u64),
}

pub struct Game {
    players: std::collections::HashMap<u64, Player>,
    status: GameStatus,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
enum Symbols {
    Cross,
    Circle,
}

pub struct GameField {
    field: [Option<Symbols>; 9],
}

enum ConvertError {
    InvalidInput,
    InvalidRange,
}

enum InvalidMove {
    InvalidInput,
    InvalidRange,
    AlreadyUsed,
}

enum ValidMove {
    Continue,
    Draw,
    Win,
}

fn get_check_neco(coordinate: &[usize; 2], field: [[Option<u64>; 3]; 3]) -> bool {
    // Check row
    let mut row_iter = field.get(coordinate[0]).unwrap().iter();
    // let row_first_item = row_iter.next().unwrap();

    match row_iter.next().unwrap() {
        Some(x) => {
            if row_iter.all(|item| item == &Some(x.to_owned())) {
                return true;
            }
        }
        None => (),
    }

    let mut a = field.iter().nth(coordinate[1]).unwrap().iter();

    let b = a.next().unwrap();

    false
}

async fn send_welcome_player(player: &Player) {
    player
        .send_msg_to_player(format!("Welcome player {}", player.get_id()))
        .await;
}

async fn send_waiting_for_another_player(player: &Player) {
    player
        .send_msg_to_player("We are waiting fro another player".to_string())
        .await;
}

async fn send_both_players_ready(player: &Player) {
    player
        .send_msg_to_player("Both players are ready".to_string())
        .await;
}

async fn send_players_leave_game(player: &Player) {
    player
        .send_msg_to_player("Another player leave game.\nCongratulation you win!".to_string())
        .await;
}

async fn send_player_move(player: &Player) {
    player
        .send_msg_to_player("You are on move".to_string())
        .await;
}

async fn send_player_wait(player: &Player) {
    player
        .send_msg_to_player("Your opponent is on move".to_string())
        .await;
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: std::collections::HashMap::new(),
            status: GameStatus::WaitingForPlayers,
        }
    }

    pub async fn add_player(&mut self, player: Player) -> bool {
        // All players connected
        if self.players.len() >= 2 {
            return false;
        }

        send_welcome_player(&player).await;

        // First player connected
        if self.players.len() == 0 {
            send_waiting_for_another_player(&player).await;

            self.players.insert(player.get_id(), player);
            return true;
        }

        self.players.insert(player.get_id(), player);

        // Both players connected
        for player in self.players.values() {
            send_both_players_ready(player).await;
        }

        let mut a = self.players.keys();

        self.status =
            GameStatus::Playing(a.next().unwrap().to_owned(), a.next().unwrap().to_owned());

        if let GameStatus::Playing(x, y) = self.status {
            send_player_move(&self.players[&x]).await;
            send_player_wait(&self.players[&y]).await;
        }

        return true;
    }

    pub async fn remove_player(&mut self, player_id: u64) {
        self.players.remove(&player_id);

        println!("ccoc");

        if self.players.len() == 1 {
            send_players_leave_game(self.players.values().next().unwrap()).await;
        }
    }
}
