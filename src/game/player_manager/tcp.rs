use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::Msg;

pub struct NewPlayerData {
    stream: tokio::net::TcpStream,
}

pub struct Player {
    name: super::super::PlayerName,
    tx_client: tokio::sync::mpsc::Sender<String>,
    last_msg_ends_with_new_line: bool,
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
            last_msg_ends_with_new_line: true,
        }
    }
}

#[async_trait::async_trait]
impl<T> super::PlayerTrait<T> for Player {
    async fn send_msg_to_player(&mut self, msg: super::MsgToPlayer<'_, T>)
    where
        T: std::fmt::Display + std::marker::Sync,
    {
        let mut text = match msg {
            super::MsgToPlayer::WelcomePlayer => "Welcome player\r\n".to_owned(),
            super::MsgToPlayer::WaitingForOtherPlayer => {
                "We are waiting for another player\r\n".to_owned()
            }
            super::MsgToPlayer::PlayersAreReady => "Both players are ready\r\n\r\n".to_owned(),
            super::MsgToPlayer::OtherPlayerLeave => {
                "Other player leave game. Congratulation you win!\r\n".to_owned()
            }
            super::MsgToPlayer::YourAreOnMove => "Now you are on move: ".to_owned(),
            super::MsgToPlayer::OtherPlayerIsOnMove => "Now is other player on move\r\n".to_owned(),
            super::MsgToPlayer::InvalidInput => "We are waiting for another player\r\n".to_owned(),
            super::MsgToPlayer::AlreadyTaken => {
                "Required field is already taken. Please repeat your input: ".to_owned()
            }
            super::MsgToPlayer::YouAreNotOnMove => {
                "You are not on move. Please wait till other player move\r\n".to_owned()
            }

            super::MsgToPlayer::YouWon => "Congratulation, you win.\r\n".to_owned(),
            super::MsgToPlayer::YouLose => "Unfortunately you lose.\r\n".to_owned(),
            super::MsgToPlayer::Draw => "Nobody win\r\n".to_owned(),
            super::MsgToPlayer::Field(T) => {
                format!("------------------\r\nCurrent game field\r\n\r\n{}\r\n", T)
            }
        };

        // Add player name prefix
        match msg {
            super::MsgToPlayer::Field(_) => (),
            _ => {
                text = format!(
                    "[{}] {}",
                    <Player as super::PlayerTrait<T>>::get_name(self),
                    text
                )
            }
        }
        match msg {
            super::MsgToPlayer::YourAreOnMove => {
                self.last_msg_ends_with_new_line = false;
            }
            super::MsgToPlayer::OtherPlayerLeave => {
                if !self.last_msg_ends_with_new_line {
                    text = format!("\r\n{}", text);
                }
            }
            _ => self.last_msg_ends_with_new_line = true,
        }

        self.tx_client.send(text).await;
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
impl<T> super::PlayerMangerTrait<T> for PlayerManager {
    type NewPlayerData = NewPlayerData;
    type Player<'a> = Player;

    fn create_new_player<'a>(
        &self,
        player_name: super::super::PlayerName,
        player_data: Self::NewPlayerData,
    ) -> Self::Player<'a> {
        Player::new(player_name, player_data.stream, self.tx.clone())
    }

    async fn receive_new_message(&mut self) -> Msg<Self::NewPlayerData> {
        return self.rx.recv().await.unwrap();
    }
}
