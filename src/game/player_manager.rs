/// Module for player manager traits
pub mod tcp;

/// Possible messages sended to player
pub enum MsgToPlayer<T> {
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
    Playboard(T),
}

/// Trait for player struct
#[async_trait::async_trait]
pub trait PlayerTrait {
    type FieldRepresentation;

    /// Get player id
    fn get_player_id(&self) -> super::PlayerId;

    /// Send new message to player
    async fn send_msg_to_player(&mut self, msg: MsgToPlayer<Self::FieldRepresentation>);
}

/// Possible messages sended from player
pub enum MsgFromPlayer<T, O> {
    /// New player joined
    Join(T),
    /// Message from player
    Msg(super::PlayerId, O),
    /// Player leave
    Leave(super::PlayerId),
}

/// Trait for player manager struct
#[async_trait::async_trait]
pub trait PlayerManagerTrait {
    /// Struct containing necessary data for crating new player
    type NewPlayerData;
    /// Struct returned by create_new_player
    type NewPlayer: PlayerTrait;

    type PlayerMsg;

    /// Creating new player from player_data
    fn create_new_player(
        &self,
        player_id: super::PlayerId,
        player_data: Self::NewPlayerData,
    ) -> Self::NewPlayer;

    /// Read new message from players
    async fn receive_new_message(&mut self) -> MsgFromPlayer<Self::NewPlayerData, Self::PlayerMsg>;
}
