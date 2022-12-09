/// Module for player manager traits
pub mod tcp;

/// Possible messages sended to player
pub enum MsgToPlayer<'a, T>
where
    T: std::fmt::Display,
{
    /// Welcome to new player
    WelcomePlayer,
    /// Waiting for another player
    WaitingForOtherPlayer,
    /// Both players are ready
    PlayersAreReady,
    /// Other player leave game
    OtherPlayerLeave,
    /// You are on move
    YourAreOnMove,
    /// Other player is on move
    OtherPlayerIsOnMove,
    /// Player use invalid input
    InvalidInput,
    /// Field is already taken
    AlreadyTaken,
    /// You are not on move
    YouAreNotOnMove,
    /// You win
    YouWon,
    /// You lose
    YouLose,
    /// Draw
    Draw,
    /// Send playboard
    Playboard(&'a T),
}

/// Trait for player struct
#[async_trait::async_trait]
pub trait PlayerTrait<T> {
    /// Get player name
    fn get_name(&self) -> super::PlayerName;

    /// Send new message to player
    async fn send_msg_to_player(&mut self, msg: MsgToPlayer<'_, T>)
    where
        T: std::fmt::Display + std::marker::Sync;
}

/// Possible messages sended from player
pub enum MsgFromPlayer<T> {
    /// New player joined
    Join(T),
    /// Message from player
    Msg(super::PlayerName, String),
    /// Player leave
    Leave(super::PlayerName),
}

/// Trait for player manager struct
#[async_trait::async_trait]
pub trait PlayerMangerTrait<T> {
    /// Struct containing necessary data for crating new player
    type NewPlayerData;
    /// Struct returned by create_new_player
    type NewPlayer<'a>: PlayerTrait<T>;

    /// Creating new player from player_data
    fn create_new_player<'a>(
        &self,
        player_name: super::PlayerName,
        player_data: Self::NewPlayerData,
    ) -> Self::NewPlayer<'a>;

    /// Read new message from players
    async fn receive_new_message(&mut self) -> MsgFromPlayer<Self::NewPlayerData>;
}
