use game_loop::db::dbclient::{DbClient, MONGO_URI};
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let uri = &MONGO_URI;
    let db_client = DbClient::new(uri).await?;

    db_client.reset_players().await.unwrap();
    db_client.reset_game_stats().await.unwrap();
    Ok(())
}
