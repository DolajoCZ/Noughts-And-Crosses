use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::Msg;

pub struct NewPlayerData {
    stream: tokio::net::TcpStream,
}

pub struct Player {
    name: super::super::PlayerName,
    tx_client: tokio::sync::mpsc::Sender<String>,
}

async fn player_communication(
    player_name: super::super::PlayerName,
    mut stream: tokio::net::TcpStream,
    tx_game: tokio::sync::mpsc::Sender<Msg<NewPlayerData>>,
    mut rx_client: tokio::sync::mpsc::Receiver<String>,
) {
    let (reader, mut writer) = stream.split();

    let mut buff = tokio::io::BufReader::new(reader);

    let mut line = String::new();

    loop {
        tokio::select! {
            // Reading input from client
            msg_length = buff.read_line(&mut line) => {
                // Connection closed
                if msg_length.unwrap() == 0 {
                    tx_game.send(Msg::Disconnect(player_name)).await;
                    return ;
                }
                // Send message from client
                tx_game.send(Msg::FromClient(player_name, line[..line.len() - 1].to_owned())).await;
                line.clear();
            }
            // Sending message to client
            msg = rx_client.recv() => {
                writer.write_all(msg.unwrap().as_bytes()).await;
            }
        }
    }
}

impl Player {
    pub fn new(
        player_name: super::super::PlayerName,
        stream: tokio::net::TcpStream,
        tx_game: tokio::sync::mpsc::Sender<Msg<NewPlayerData>>,
    ) -> Player {
        let (tx_client, rx_client) = tokio::sync::mpsc::channel(5);

        tokio::spawn(player_communication(
            player_name,
            stream,
            tx_game,
            rx_client,
        ));

        Player {
            name: player_name,
            tx_client,
        }
    }
}

#[async_trait::async_trait]
impl super::PlayerTrait for Player {
    async fn send_msg_to_player(&self, msg: String) {
        self.tx_client.send(msg).await;
    }

    fn get_name(&self) -> super::super::PlayerName {
        self.name
    }
}

pub struct PlayerManager {
    tx: tokio::sync::mpsc::Sender<super::Msg<NewPlayerData>>,
    rx: tokio::sync::mpsc::Receiver<super::Msg<NewPlayerData>>,
}

async fn connection_listener(
    listener: tokio::net::TcpListener,
    tx: tokio::sync::mpsc::Sender<super::Msg<NewPlayerData>>,
) {
    loop {
        let (stream, address) = listener.accept().await.unwrap();

        tx.send(super::Msg::NewConnection(NewPlayerData { stream: stream }))
            .await;
    }
}

impl PlayerManager {
    pub async fn new<T>(addr: T) -> Self
    where
        T: tokio::net::ToSocketAddrs,
    {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(x) => x,
            Err(err) => {
                println!("Fail to create new TCP listener: {:?}", err);
                std::process::exit(2);
            }
        };

        let (tx, rx) = tokio::sync::mpsc::channel(10);

        tokio::spawn(connection_listener(listener, tx.clone()));

        PlayerManager { tx, rx }
    }
}

#[async_trait::async_trait]
impl super::PlayerMangerTrait for PlayerManager {
    type NewPlayerData = NewPlayerData;
    type Player = Player;

    fn create_new_player(
        &self,
        player_name: super::super::PlayerName,
        player_data: Self::NewPlayerData,
    ) -> Self::Player {
        Player::new(player_name, player_data.stream, self.tx.clone())
    }

    async fn receive_new_message(&mut self) -> Msg<Self::NewPlayerData> {
        return self.rx.recv().await.unwrap();
    }
}
