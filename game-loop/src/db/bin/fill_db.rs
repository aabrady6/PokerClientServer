use game_loop::db::dbclient::{DbClient, MONGO_URI};
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
    let player_ids = vec![17, 18, 19, 20, 21, 22, 23, 24, 25, 26];
    let player_money = vec![1000, 2000, 3000, 4000, 5000, 6000, 5000, 4000, 3000, 2000];
    let games = vec![10, 100, 1000, 20, 200, 30, 300, 40, 400, 50];
    let wins = vec![5, 20, 800, 0, 200, 10, 200, 25, 100, 48];
    let earnings = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 0];

    let uri = &MONGO_URI;
    let db_client = DbClient::new(uri).await?;

    for (index, name) in player_names.iter().enumerate() {
        let mut created_player = Player::new(&name);
        created_player.set_id(player_ids[index]);
        created_player.set_money(player_money[index]);
        created_player.set_games(games[index]);
        created_player.set_wins(wins[index]);
        created_player.set_losses(games[index] - wins[index]);
        created_player.set_earnings(earnings[index]);
        db_client.insert(&created_player).await?;
    }

    Ok(())
}
