/// Module for playboard traits
pub mod pb_n_n;

/// Enum for invalid operation on playboard
pub enum InvalidMove {
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
    type Position;

    /// Make new move on playboard
    fn new_move(
        &mut self,
        position: Self::Position,
        player_id: super::PlayerId,
    ) -> Result<ValidMove, InvalidMove>;
}
