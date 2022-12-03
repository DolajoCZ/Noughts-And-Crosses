// pub mod msgs;
pub mod tcp;

pub enum MsgToPlayer<'a, T>
where
    T: std::fmt::Display,
{
    WelcomePlayer,
    WaitingForOtherPlayer,
    PlayersAreReady,
    OtherPlayerLeave,
    YourAreOnMove,
    OtherPlayerIsOnMove,
    InvalidInput,
    AlreadyTaken,
    YouAreNotOnMove,
    YouWon,
    YouLose,
    Draw,
    Field(&'a T),
}

#[async_trait::async_trait]
pub trait PlayerTrait<T> {
    fn get_name(&self) -> super::PlayerName;
    async fn send_msg_to_player(&mut self, msg: MsgToPlayer<'_, T>)
    where
        T: std::fmt::Display + std::marker::Sync;
}

pub enum Msg<T> {
    NewConnection(T),
    FromClient(super::PlayerName, String),
    Disconnect(super::PlayerName),
}

#[async_trait::async_trait]
pub trait PlayerMangerTrait<T> {
    type NewPlayerData;
    type Player<'a>: PlayerTrait<T>;

    fn create_new_player<'a>(
        &self,
        player_name: super::PlayerName,
        player_data: Self::NewPlayerData,
    ) -> Self::Player<'a>;

    async fn receive_new_message(&mut self) -> Msg<Self::NewPlayerData>;
}
