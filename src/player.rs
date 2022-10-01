use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub struct Player {
    id: u64,
    tx_client: tokio::sync::mpsc::Sender<String>,
}

async fn client_communication(
    mut stream: tokio::net::TcpStream,
    id: u64,
    tx_game: tokio::sync::mpsc::Sender<crate::Msg>,
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
                    tx_game.send(crate::Msg::Disconnect(id)).await;
                    return ;
                }
                // Send message from client
                tx_game.send(crate::Msg::FromClient(id, line.clone())).await;
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
        id: u64,
        tx_game: tokio::sync::mpsc::Sender<crate::Msg>,
    ) -> Player {
        let (tx_client, rx_client) = tokio::sync::mpsc::channel(5);

        tokio::spawn(client_communication(stream, id, tx_game, rx_client));

        Player { id, tx_client }
    }

    pub async fn send_msg_to_player(&self, msg: String) {
        self.tx_client.send(msg).await;
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}
