use crate::db::dbclient::{DbClient, MONGO_URI};
use crate::game::player::Player;

/// Logs in a player by verifying their username and password.
///
/// # Arguments
///
/// * `username` - A `String` containing the player's username.
/// * `password` - A `String` containing the player's password.
///
/// # Returns
///
/// * `Ok(Player)` - If authentication is successful.
/// * `Err(String)` - If authentication fails due to incorrect credentials or missing player.
///
/// # Errors
///
/// Returns an error if the password is incorrect or the player is not found.
///
/// # Example
///
/// ```rust
/// let result = login_player("player1".to_string(), "password123".to_string()).await;
/// match result {
///     Ok(player) => println!("Welcome, {}!", player.player_name),
///     Err(e) => println!("Login failed: {}", e),
/// }
/// ```
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
