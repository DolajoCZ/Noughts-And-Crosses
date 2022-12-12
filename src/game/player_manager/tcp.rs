use std::io::Write;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::MsgFromPlayer;

fn get_available_interfaces() -> Result<Vec<(String, std::net::IpAddr)>, ()> {
    let network_interfaces = local_ip_address::list_afinet_netifas().map_err(|err| {
        log::error!("Unable to load network interfaces: {}", err);
        ()
    })?;

    log::info!("Available interfaces are:");
    for (index, (name, addr)) in network_interfaces.iter().enumerate() {
        log::info!("Id: {} - Name: {} - Ip address: {}", index, name, addr);
    }

    Ok(network_interfaces)
}

fn get_user_input() -> Result<usize, ()> {
    log::info!("Please select interface Id: ");
    std::io::stdout().flush().map_err(|err| {
        log::error!("Unable to read user input: {}", err);
        ()
    })?;

    // Read user input
    let mut user_input = String::new();

    let length = std::io::stdin().read_line(&mut user_input).map_err(|err| {
        log::error!("Unable to read user input: {}", err);
        ()
    })?;

    let user_input = &user_input[..length];

    let user_input = user_input
        .strip_suffix("\r\n")
        .unwrap_or(user_input.strip_suffix("\n").unwrap_or(user_input));

    user_input.parse().map_err(|_| {
        log::error!(
            "Unable to convert \"{}\" to integer in range [0 - {}]",
            &user_input,
            usize::MAX
        );
        ()
    })
}

pub fn select_network() -> Result<std::net::IpAddr, ()> {
    let interfaces = get_available_interfaces()?;
    let index = get_user_input()?;

    match interfaces.get(index) {
        Some(x) => Ok(x.1),
        None => {
            log::error!("Used index is out of options");
            Err(())
        }
    }
}

pub struct NewPlayerData {
    stream: tokio::net::TcpStream,
}

pub struct Player {
    id: super::super::PlayerId,
    tx_client: tokio::sync::mpsc::Sender<String>,
    last_msg_ends_with_new_line: bool,
}

async fn player_communication(
    player_id: super::super::PlayerId,
    mut stream: tokio::net::TcpStream,
    tx_game: tokio::sync::mpsc::Sender<MsgFromPlayer<NewPlayerData>>,
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
                    tx_game.send(MsgFromPlayer::Leave(player_id)).await;
                    return ;
                }
                // Send message from client
                tx_game.send(MsgFromPlayer::Msg(player_id, line[..line.len() - 1].to_owned())).await;
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
        player_id: super::super::PlayerId,
        stream: tokio::net::TcpStream,
        tx_game: tokio::sync::mpsc::Sender<MsgFromPlayer<NewPlayerData>>,
    ) -> Player {
        let (tx_client, rx_client) = tokio::sync::mpsc::channel(5);

        tokio::spawn(player_communication(player_id, stream, tx_game, rx_client));

        Player {
            id: player_id,
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
            super::MsgToPlayer::InvalidInput => {
                "You pass invalid input. Please repeat your input: ".to_owned()
            }
            super::MsgToPlayer::AlreadyTaken => {
                "Required field is already taken. Please repeat your input: ".to_owned()
            }
            super::MsgToPlayer::YouAreNotOnMove => {
                "You are not on move. Please wait till other player move\r\n".to_owned()
            }

            super::MsgToPlayer::YouWon => "Congratulation, you win.\r\n".to_owned(),
            super::MsgToPlayer::YouLose => "Unfortunately you lose.\r\n".to_owned(),
            super::MsgToPlayer::Draw => "Nobody win\r\n".to_owned(),
            super::MsgToPlayer::Playboard(T) => {
                format!("------------------\r\nCurrent game field\r\n\r\n{}\r\n", T)
            }
        };

        // Add player name prefix
        match msg {
            super::MsgToPlayer::Playboard(_) => (),
            _ => {
                text = format!(
                    "[{}] {}",
                    <Player as super::PlayerTrait<T>>::get_player_id(self),
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

    fn get_player_id(&self) -> super::super::PlayerId {
        self.id
    }
}

pub struct PlayerManager {
    tx: tokio::sync::mpsc::Sender<super::MsgFromPlayer<NewPlayerData>>,
    rx: tokio::sync::mpsc::Receiver<super::MsgFromPlayer<NewPlayerData>>,
}

async fn connection_listener(
    listener: tokio::net::TcpListener,
    tx: tokio::sync::mpsc::Sender<super::MsgFromPlayer<NewPlayerData>>,
) {
    loop {
        let (stream, address) = listener.accept().await.unwrap();

        tx.send(super::MsgFromPlayer::Join(NewPlayerData { stream: stream }))
            .await;
    }
}

async fn get_available_listener(
    ip: std::net::IpAddr,
) -> tokio::io::Result<tokio::net::TcpListener> {
    for port in 49152..65535 {
        let socket_addr = std::net::SocketAddr::new(ip, port);

        match tokio::net::TcpListener::bind(socket_addr).await {
            Ok(x) => return Ok(x),
            Err(err) => match err.kind() {
                std::io::ErrorKind::AddrInUse => continue,
                _ => return Err(err),
            },
        }
    }

    Err(std::io::Error::from(std::io::ErrorKind::AddrInUse))
}

impl PlayerManager {
    pub fn new(listener: tokio::net::TcpListener) -> Self {
        log::info!(
            "TCP player manager running on address: {}",
            listener.local_addr().unwrap()
        );

        let (tx, rx) = tokio::sync::mpsc::channel(10);

        tokio::spawn(connection_listener(listener, tx.clone()));

        PlayerManager { tx, rx }
    }

    pub async fn from_ip(ip: std::net::IpAddr) -> Result<PlayerManager, std::io::Error> {
        let listener = get_available_listener(ip).await?;
        Ok(Self::new(listener))
    }

    pub async fn from_socket_address<T>(addr: T) -> Result<PlayerManager, std::io::Error>
    where
        T: tokio::net::ToSocketAddrs,
    {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        Ok(Self::new(listener))
    }
}

#[async_trait::async_trait]
impl<T> super::PlayerManagerTrait<T> for PlayerManager {
    type NewPlayerData = NewPlayerData;
    type NewPlayer<'a> = Player;

    fn create_new_player<'a>(
        &self,
        player_id: super::super::PlayerId,
        player_data: Self::NewPlayerData,
    ) -> Self::NewPlayer<'a> {
        Player::new(player_id, player_data.stream, self.tx.clone())
    }

    async fn receive_new_message(&mut self) -> MsgFromPlayer<Self::NewPlayerData> {
        return self.rx.recv().await.unwrap();
    }
}
