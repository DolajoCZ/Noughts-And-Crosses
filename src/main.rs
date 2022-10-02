mod game;

const ADDRESS: &str = "localhost:8000";

#[tokio::main]
async fn main() {
    game::run(ADDRESS).await;
}
