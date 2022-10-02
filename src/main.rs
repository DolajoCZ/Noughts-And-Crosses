// mod field;
// mod game;
mod game;
// mod player;

const ADDRESS: &str = "localhost:8000";

// pub enum Msg {
//     NewConnection(std::net::SocketAddr, tokio::net::TcpStream),
//     FromClient(u64, String),
//     Disconnect(u64),
// }

// async fn connections_manager(
//     listener: tokio::net::TcpListener,
//     tx: tokio::sync::mpsc::Sender<Msg>,
// ) {
//     loop {
//         let (stream, address) = listener.accept().await.unwrap();
//         tx.send(Msg::NewConnection(address, stream)).await;
//     }
// }

// async fn game(
//     tx_game: tokio::sync::mpsc::Sender<Msg>,
//     mut rx_game: tokio::sync::mpsc::Receiver<Msg>,
// ) {
//     let mut game = game::Game::new();
//     let mut id = 1_u64;

//     loop {
//         match rx_game.recv().await.unwrap() {
//             // Connected new player
//             Msg::NewConnection(_, mut stream) => {
//                 game.add_player(player::Player::new(stream, id, tx_game.clone()))
//                     .await;

//                 id += 1;
//             }
//             // Message from client
//             Msg::FromClient(id, msg) => println!("Message from user {}: {}", id, msg),

//             // Client disconnected
//             Msg::Disconnect(id) => game.remove_player(id).await,
//         };
//     }
// }

#[tokio::main]
async fn main() {
    game::run(ADDRESS).await;

    // let listener = match tokio::net::TcpListener::bind(ADDRESS).await {
    //     Ok(x) => x,
    //     Err(err) => {
    //         println!("Fail to create new TCP listener: {:?}", err);
    //         std::process::exit(2);
    //     }
    // };

    // let (tx, rx) = tokio::sync::mpsc::channel(10);

    // tokio::join!(connections_manager(listener, tx.clone()), game(tx, rx));

    println!("Hello, world!");
}
