use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub struct Player {
    name: super::PlayerName,
    tx_client: tokio::sync::mpsc::Sender<String>,
}

async fn client_communication(
    mut stream: tokio::net::TcpStream,
    name: super::PlayerName,
    tx_game: tokio::sync::mpsc::Sender<super::Msg>,
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
                    tx_game.send(super::Msg::Disconnect(name)).await;
                    return ;
                }
                // Send message from client
                tx_game.send(super::Msg::FromClient(name, line[..line.len() - 1].to_owned())).await;
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
        stream: tokio::net::TcpStream,
        name: super::PlayerName,
        tx_game: tokio::sync::mpsc::Sender<super::Msg>,
    ) -> Player {
        let (tx_client, rx_client) = tokio::sync::mpsc::channel(5);

        tokio::spawn(client_communication(stream, name, tx_game, rx_client));

        Player { name, tx_client }
    }
}

#[async_trait::async_trait]
impl super::Player for Player {
    type T = super::PlayerName;

    fn get_name(&self) -> Self::T {
        self.name
    }

    async fn send_msg_to_player(&self, msg: String) {
        self.tx_client.send(msg).await;
    }
}
