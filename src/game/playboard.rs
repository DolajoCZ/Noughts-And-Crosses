pub mod bp_3_3;

// ---- Trait for playboard ----
pub enum InvalidMove {
    InvalidInput,
    InvalidRange,
    AlreadyUsed,
}

pub enum ValidMove {
    Continue,
    Draw,
    Win,
}

pub trait Playboard {
    fn new_move(
        &mut self,
        input: &str,
        player_name: super::PlayerName,
    ) -> Result<ValidMove, InvalidMove>;
}
