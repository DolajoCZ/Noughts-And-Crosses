/// Module for playboard traits
pub mod bp_3_3;

/// Enum for invalid operation on playboard
pub enum InvalidMove {
    /// Input not possible to convert to playboard coordinates
    InvalidInput,
    /// Input out of playboard coordinates
    InvalidRange,
    /// Field on playboard is already used
    AlreadyUsed,
}

/// Enum for valid operation on playboard
pub enum ValidMove {
    /// Move without game finish
    Continue,
    /// Move where game ended an nobody win
    Draw,
    /// Move where player win
    Win,
}

/// Trait for playboard struct
pub trait Playboard {
    /// Make new move on playboard
    fn new_move(
        &mut self,
        input: &str,
        player_name: super::PlayerName,
    ) -> Result<ValidMove, InvalidMove>;
}
