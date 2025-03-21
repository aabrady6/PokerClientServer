use game_loop::db::dbclient::DbClient;
use game_loop::game::player::Player;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let player_names = vec![
        "fake_gene",
        "fake_frank",
        "fake_alice",
        "fake_brian",
        "fake_sam",
        "fake_wanda",
        "fake_sarah",
        "fake_betty",
        "fake_betsy",
        "fake_vinny",
    ];

    let uri = "mongodb://localhost:27017";
    let db_client = DbClient::new(uri).await?;

    for (_index, name) in player_names.iter().enumerate() {
        let player = Player::new(&name);
        db_client.delete_one(&player).await.unwrap();
    }

    Ok(())
}
