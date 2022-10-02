use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::super::game;
use super::field;

pub struct Player {
    name: field::Symbols,
    tx_client: tokio::sync::mpsc::Sender<String>,
}

async fn client_communication(
    mut stream: tokio::net::TcpStream,
    name: field::Symbols,
    tx_game: tokio::sync::mpsc::Sender<game::Msg>,
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
                    tx_game.send(game::Msg::Disconnect(name)).await;
                    return ;
                }
                // Send message from client
                tx_game.send(game::Msg::FromClient(name, line[..line.len() - 1].to_owned())).await;
                line.clear();
            }
            // Sending message to client
            msg = rx_client.recv() => {
                let mut a = msg.unwrap();
                a.push_str("\n");

                writer.write_all(a.as_bytes()).await;
            }
        }
    }
}

impl Player {
    pub fn new(
        stream: tokio::net::TcpStream,
        name: field::Symbols,
        tx_game: tokio::sync::mpsc::Sender<game::Msg>,
    ) -> Player {
        let (tx_client, rx_client) = tokio::sync::mpsc::channel(5);

        tokio::spawn(client_communication(stream, name, tx_game, rx_client));

        Player { name, tx_client }
    }

    pub async fn send_msg_to_player(&self, msg: String) {
        self.tx_client.send(msg).await;
    }

    pub fn get_name(&self) -> field::Symbols {
        return self.name;
    }
}
