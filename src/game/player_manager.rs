pub mod msgs;
pub mod tcp;

#[async_trait::async_trait]
pub trait PlayerTrait {
    fn get_name(&self) -> super::PlayerName;
    async fn send_msg_to_player(&self, msg: String);
}

pub enum Msg<T> {
    NewConnection(T),
    FromClient(super::PlayerName, String),
    Disconnect(super::PlayerName),
}

#[async_trait::async_trait]
pub trait PlayerMangerTrait {
    type NewPlayerData;
    type Player: PlayerTrait;

    fn create_new_player(
        &self,
        player_name: super::PlayerName,
        player_data: Self::NewPlayerData,
    ) -> Self::Player;

    async fn receive_new_message(&mut self) -> Msg<Self::NewPlayerData>;
}
