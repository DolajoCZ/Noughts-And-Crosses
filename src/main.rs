use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

const ADDRESS: &str = "localhost:8000";

enum Msg {
    NewConnection(std::net::SocketAddr, tokio::net::TcpStream),
    FromClient(u64, String),
    Disconnect(u64),
}

async fn connections_manager(
    listener: tokio::net::TcpListener,
    tx: tokio::sync::mpsc::Sender<Msg>,
) {
    loop {
        let (stream, address) = listener.accept().await.unwrap();
        tx.send(Msg::NewConnection(address, stream)).await;
    }
}

async fn client_communication(
    mut stream: tokio::net::TcpStream,
    id: u64,
    tx_game: tokio::sync::mpsc::Sender<Msg>,
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
                    tx_game.send(Msg::Disconnect(id)).await;
                    return ;
                }
                // Send message from client
                tx_game.send(Msg::FromClient(id, line.clone())).await;
                line.clear();
            }
            // Sending message to client
            msg = rx_client.recv() => {
                writer.write_all(msg.unwrap().as_bytes()).await;
            }
        }
    }
}

async fn game(
    tx_game: tokio::sync::mpsc::Sender<Msg>,
    mut rx_game: tokio::sync::mpsc::Receiver<Msg>,
) {
    let mut players = std::collections::HashMap::new();

    let mut id = 1_u64;

    loop {
        match rx_game.recv().await.unwrap() {
            // Connected new player
            Msg::NewConnection(_, stream) => {
                // Players already connected
                if players.len() == 2 {
                    println!("Currently are all players connected.");
                } else {
                    let (tx_client, rx_client) = tokio::sync::mpsc::channel(5);

                    players.insert(id, tx_client);

                    tokio::spawn(client_communication(stream, id, tx_game.clone(), rx_client));

                    players[&id]
                        .send(format!("Welcome to new game player {}\n", id))
                        .await;

                    if players.len() == 1 {
                        players[&id]
                            .send("Waiting for another player\n".to_string())
                            .await;
                    } else {
                        for (_, tx_client) in players.iter() {
                            tx_client
                                .send("Both player are connected\n".to_string())
                                .await;
                        }
                    }

                    id += 1;
                }
            }
            // Message from client
            Msg::FromClient(id, msg) => println!("Message from user {}: {}", id, msg),

            // Client disconnected
            Msg::Disconnect(id) => {
                players.remove(&id);

                match players.keys().next() {
                    Some(x) => {
                        players[x]
                            .send("Another player leave game.\n".to_string())
                            .await;
                    }
                    _ => (),
                }
            }
            _ => println!("Other operation"),
        };
    }
}

#[tokio::main]
async fn main() {
    let listener = match tokio::net::TcpListener::bind(ADDRESS).await {
        Ok(x) => x,
        Err(err) => {
            println!("Fail to create new TCP listener: {:?}", err);
            std::process::exit(2);
        }
    };

    let (tx, rx) = tokio::sync::mpsc::channel(10);

    tokio::join!(connections_manager(listener, tx.clone()), game(tx, rx));

    println!("Hello, world!");
}
