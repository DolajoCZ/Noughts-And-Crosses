use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

const ADDRESS: &str = "localhost:8000";

enum Msg {
    NewConnection(std::net::SocketAddr, tokio::net::TcpStream),
    FromClient(std::net::SocketAddr, String),
    Disconnect(u64),
    Msg(String),
}

struct Player {
    address: std::net::SocketAddr,
    stream: tokio::net::TcpStream,
    rx: tokio::sync::mpsc::Receiver<Msg>,
}

impl Player {
    fn new(
        address: std::net::SocketAddr,
        stream: tokio::net::TcpStream,
    ) -> (Player, tokio::sync::mpsc::Sender<Msg>) {
        let (tx, rx) = tokio::sync::mpsc::channel(5);

        (
            Player {
                address,
                stream,
                rx,
            },
            tx,
        )
    }
}

async fn connections_manager(
    listener: tokio::net::TcpListener,
    tx: tokio::sync::mpsc::Sender<Msg>,
) {
    loop {
        println!("Waiting for new connection");
        let (stream, address) = listener.accept().await.unwrap();
        println!("New connection found");
        tx.send(Msg::NewConnection(address, stream)).await;
    }
}

async fn kkk(
    mut stream: tokio::net::TcpStream,
    id: u64,
    tx: tokio::sync::mpsc::Sender<Msg>,
    mut rx: tokio::sync::mpsc::Receiver<String>,
) {
    let (reader, mut writer) = stream.split();

    let mut buff = tokio::io::BufReader::new(reader);

    let mut line = String::new();

    loop {
        tokio::select! {
            msg_length = buff.read_line(&mut line) => {
                println!("dddd");
                if msg_length.unwrap() == 0 {


                    let a = tx.send(Msg::Disconnect(id)).await;

                    // match a {
                    //     Ok(()) => println!("Ok"),
                    //     Err(e) => println!("Err")
                    // }

                    // tx.send(Msg::Msg("sdfd".to_string()));
                    return ;
                }
                tx.send(Msg::Msg(line.clone())).await;

                // match a {
                //     Ok()
                // }
                // println!("{:?}", );
                line.clear();

            }
            msg = rx.recv() => {
                writer.write_all(msg.unwrap().as_bytes()).await;
            }
        }
    }
}

async fn game(tx: tokio::sync::mpsc::Sender<Msg>, mut rx: tokio::sync::mpsc::Receiver<Msg>) {
    let mut players = std::collections::HashMap::new();

    let mut id = 1_u64;

    loop {
        println!("Waiting for new msg");
        match rx.recv().await.unwrap() {
            Msg::NewConnection(addr, stream) => {
                if players.len() == 2 {
                    println!("Currently are all players connected.");
                } else {
                    let (str_tx, str_rx) = tokio::sync::mpsc::channel(5);

                    // let x = str_tx.clone();

                    players.insert(id, str_tx);

                    let xxx = tx.clone();

                    tokio::spawn(kkk(stream, id, xxx, str_rx));

                    id += 1;
                }
            }
            Msg::Disconnect(id) => {
                println!("dddddddddd");

                println!("{}", players.contains_key(&id));
                print!("Disc");
            }
            _ => println!("Other operation"),
        };
        println!("Connected");
    }
}

async fn neco(tx: tokio::sync::mpsc::Sender<Msg>, time: u64, id: u8) {
    for _ in 0..11 {
        println!("Id: {} - sleep: {}", id, time);
        tx.send(Msg::Msg("Neoc".to_string())).await;
        tokio::time::sleep(std::time::Duration::from_secs(time)).await;
    }
}

async fn dva(mut rx: tokio::sync::mpsc::Receiver<Msg>) {
    while let Some(x) = rx.recv().await {
        match x {
            Msg::Msg(k) => println!("Received data: {}", k),
            _ => (),
        }
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

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let x = tx.clone();

    tokio::join!(connections_manager(listener, tx), game(x, rx));

    println!("Hello, world!");
}
