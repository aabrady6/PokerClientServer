use crate::db::dbclient::{DbClient, MONGO_URI};
use crate::game::player::Player;

pub async fn login_player(username: String, password: String) -> Result<Player, String> {
    let db_client = DbClient::new(&MONGO_URI).await.unwrap();

    let query_player = Player::new(&username);

    if let Some(player) = db_client.query_one(&query_player).await? {
        if player.verify_password(&password) {
            Ok(player)
        } else {
            Err("Invalid password".to_string())
        }
    } else {
        Err("Player not found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_auth_success() {
        let db_client = DbClient::new(&MONGO_URI).await.unwrap();
        assert!(db_client.ping().await.is_ok());

        let inserted_player =
            match Player::new_player("George Michael".to_string(), "hunter1".to_string(), 5654) {
                Ok(player) => player,
                Err(e) => panic!("Failed to create player: {}", e),
            };

        let _ = db_client.insert(&inserted_player).await;

        match login_player("George Michael".to_string(), "hunter1".to_string()).await {
            Ok(player) => {
                assert_eq!(player.player_name, inserted_player.player_name);
            }
            Err(e) => panic!("Error:{}", e),
        }

        let _ = db_client.delete_one(&inserted_player).await;
    }

    #[tokio::test]
    async fn test_auth_fail() {
        let db_client = DbClient::new(&MONGO_URI).await.unwrap();
        assert!(db_client.ping().await.is_ok());

        let inserted_player = match Player::new_player(
            "George Michaelson".to_string(),
            "hunter1".to_string(),
            5655,
        ) {
            Ok(player) => player,
            Err(e) => panic!("Failed to create player: {}", e),
        };

        let _ = db_client.insert(&inserted_player).await;

        match login_player("George Michaelson".to_string(), "hunter2".to_string()).await {
            Ok(_) => {
                panic!("HOW DID YOU LOGIN HAHA?!?!");
            }
            Err(e) => assert_eq!(e, "Invalid password".to_string()),
        }

        let _ = db_client.delete_one(&inserted_player).await;
    }

    #[tokio::test]
    async fn test_auth_no_player() {
        let db_client = DbClient::new(&MONGO_URI).await.unwrap();
        assert!(db_client.ping().await.is_ok());

        let inserted_player = match Player::new_player(
            "Georgeson Michaelson".to_string(),
            "hunter1".to_string(),
            5656,
        ) {
            Ok(player) => player,
            Err(e) => panic!("Failed to create player: {}", e),
        };

        let _ = db_client.insert(&inserted_player).await;

        match login_player("FakeMichaelson".to_string(), "hunter3".to_string()).await {
            Ok(_) => {
                panic!("HOW DID YOU LOGIN HAHA?!?!");
            }
            Err(e) => assert_eq!(e, "Player not found".to_string()),
        }

        let _ = db_client.delete_one(&inserted_player).await;
    }
}
