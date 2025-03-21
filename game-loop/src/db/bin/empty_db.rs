use game_loop::db::dbclient::DbClient;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let uri = "mongodb://localhost:27017";
    let db_client = DbClient::new(uri).await?;

    db_client.reset_players().await.unwrap();
    db_client.reset_game_stats().await.unwrap();
    Ok(())
}
