use futures::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOneOptions, ServerApi, ServerApiVersion},
    Client, Collection, Database,
};
use std::error::Error;

/// Client used for interacting with the MongoDB instance
pub struct DbClient {
    database: Database,
}

/// trait to interact with the DB for games and players
pub trait DbEntity: Sized {
    /// Returns the name of the MongoDB collection for this entity
    fn collection_name() -> &'static str;
    /// Converts the entity into a MongoDB document
    fn to_document(&self) -> Result<Document, String>;
    /// Converts a MongoDB document into an entity instance
    fn from_document(doc: &Document) -> Result<Self, String>;
    /// Returns a unique field filter for MongoDB queries
    fn unique_field(&self) -> Document;
}

impl DbClient {
    /// Creates a new instance of `DbClient` by connecting to the specified MongoDB uri
    ///
    /// # Arguments
    /// * `uri` - A string slice that holds the MongoDB connection uri
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn Error>>` - Returns an instance of `DbClient` on success, or an error on failure.
    ///
    /// # Example
    /// ```
    /// let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();
    /// ```
    pub async fn new(uri: &str) -> Result<Self, Box<dyn Error>> {
        let mut client_options = ClientOptions::parse(uri).await?;
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();

        client_options.server_api = Some(server_api);
        let client = Client::with_options(client_options)?;

        let database = client.database("poker-project-balotro");

        Ok(DbClient { database })
    }

    /// Pings the MongoDB server to check the connection status
    ///
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - Returns `Ok(())` if the server is reachable, or an error if the ping fails.
    ///
    /// # Example
    /// ```
    /// let result = db_client.ping().await;
    /// assert!(result.is_ok());
    /// ```
    #[cfg(test)]
    pub async fn ping(&self) -> Result<(), Box<dyn Error>> {
        let client = self.database.client();
        let db = client.database("admin");
        db.run_command(doc! {"ping": 1}).await?;
        Ok(())
    }

    /// Inserts a new entity into the MongoDB collection
    ///
    /// # Arguments
    /// * `entity` - A reference to the entity to be inserted.
    ///
    /// # Returns
    /// * `Ok(Some(entity.clone()))` if the insertion is successful
    /// * `Err(String)` if an error occurs
    pub async fn insert<T: DbEntity + Clone>(&self, entity: &T) -> Result<Option<T>, String> {
        let collection: Collection<Document> = self.database.collection(T::collection_name());

        let doc = entity.to_document()?;
        match collection.insert_one(doc).await {
            Ok(_) => Ok(Some(entity.clone())),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Updates an existing entity in the MongoDB collection
    ///
    /// # Arguments
    /// * `entity` - A reference to the entity to be updated
    ///
    /// # Returns
    /// * `Ok(())` if the update is successful
    /// * `Err(String)` if an error occurs or no matching document is found
    pub async fn update_one<T: DbEntity>(&self, entity: &T) -> Result<(), String> {
        let collection: Collection<Document> = self.database.collection(T::collection_name());

        let filter = entity.unique_field();
        let updated_doc = entity.to_document()?;

        let update_query = doc! {
            "$set": updated_doc
        };

        match collection.update_one(filter, update_query).await {
            Ok(update_result) => {
                if update_result.matched_count > 0 {
                    Ok(())
                } else {
                    Err("No matching document found".to_string())
                }
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Deletes an entity from the MongoDB collection
    ///
    /// # Arguments
    /// * `entity` - A reference to the entity to be deleted
    ///
    /// # Returns
    /// * `Ok(())` if the deletion is successful
    /// * `Err(String)` if an error occurs or no matching document is found
    // #[cfg(test)]
    #[allow(dead_code)]
    pub async fn delete_one<T: DbEntity>(&self, entity: &T) -> Result<(), String> {
        let collection: Collection<Document> = self.database.collection(T::collection_name());

        let filter = entity.unique_field();

        match collection.delete_one(filter).await {
            Ok(delete_result) => {
                if delete_result.deleted_count > 0 {
                    Ok(())
                } else {
                    Err("No matching document found to delete".to_string())
                }
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    #[allow(dead_code)]
    pub async fn reset_players(&self) -> Result<(), String> {
        let collection: Collection<Document> = self.database.collection("players");

        let filter = doc! {};

        let delete_result = collection.delete_many(filter).await;

        match delete_result {
            Ok(_) => {}
            Err(e) => return Err(format!("Error Reading: {}", e)),
        }

        Ok(())
    }

    /// Queries a single entity from the MongoDB collection based on its unique field
    ///
    /// # Arguments
    /// * `entity` - A reference to an entity with its unique field set
    ///
    /// # Returns
    /// * `Ok(Some(entity))` if the entity is found
    /// * `Ok(None)` if no entity matches the query
    /// * `Err(String)` if an error occurs
    pub async fn query_one<T: DbEntity>(&self, entity: &T) -> Result<Option<T>, String> {
        let collection: Collection<Document> = self.database.collection(T::collection_name());

        let filter = entity.unique_field();

        match collection.find_one(filter).await {
            Ok(Some(doc)) => match T::from_document(&doc) {
                Ok(entity) => Ok(Some(entity)),
                Err(e) => {
                    println!("{:?}", doc);
                    Err(format!("Error Deserializing: {}", e))
                }
            },
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Queries all entities from the MongoDB collection
    ///
    /// # Returns
    /// * `Ok(Vec<T>)` containing all retrieved entities
    /// * `Err(String)` if an error occurs during the query
    pub async fn query_all<T: DbEntity>(&self) -> Result<Vec<T>, String> {
        let collection: Collection<Document> = self.database.collection(T::collection_name());

        let filter = doc! {};

        let mut cursor = match collection.find(filter).await {
            Ok(cursor) => cursor,
            Err(e) => return Err(format!("Database error: {}", e)),
        };

        let mut entities = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(t) => match T::from_document(&t) {
                    Ok(entity) => entities.push(entity),
                    Err(e) => return Err(format!("Error Deserializing: {}", e)),
                },
                Err(e) => return Err(format!("Error Reading: {}", e)),
            }
        }

        Ok(entities)
    }

    pub async fn get_new_player_id(&self) -> u32 {
        let collection: Collection<Document> = self.database.collection("players");

        let filter = doc! {};

        let find_options = FindOneOptions::builder()
            .sort(doc! { "player_id": -1 })
            .build();

        let cursor = collection.find_one(filter).with_options(find_options).await;

        match cursor {
            Ok(Some(doc)) => match doc.get_i64("player_id") {
                Ok(game_id) => (game_id as u32) + 1,
                Err(_) => 1,
            },
            _ => 1,
        }
    }

    pub async fn get_new_game_id(&self) -> u32 {
        let collection: Collection<Document> = self.database.collection("games");

        let filter = doc! {};

        let find_options = FindOneOptions::builder()
            .sort(doc! { "game_id": -1 })
            .build();

        let cursor = collection.find_one(filter).with_options(find_options).await;

        match cursor {
            Ok(Some(doc)) => match doc.get_i64("game_id") {
                Ok(game_id) => (game_id as u32) + 1,
                Err(_) => 1,
            },
            _ => 1,
        }
    }

    pub async fn reset_game_stats(&self) -> Result<(), String> {
        let collection: Collection<Document> = self.database.collection("games");

        let filter = doc! {};

        let delete_result = collection.delete_many(filter).await;

        match delete_result {
            Ok(_) => {}
            Err(e) => return Err(format!("Error Reading: {}", e)),
        }

        Ok(())
    }

    pub async fn reset_player_stats(&self) -> Result<(), String> {
        let collection: Collection<Document> = self.database.collection("players");

        let filter = doc! {};

        let update = doc! {
            "$set": {
                "total_games": 0 as i64,
                "total_wins": 0 as i64,
                "total_losses": 0 as i64,
                "total_earnings": 0 as i64,
           }
        };

        let update_result = collection.update_many(filter, update).await;

        match update_result {
            Ok(_) => {}
            Err(e) => return Err(format!("Error Reading: {}", e)),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{base_rules::GameState, player::Player};
    use tokio;

    #[tokio::test]
    async fn test_db_client_connection_connect_and_ping() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();
        assert!(db_client.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_dbclient_insert_and_retrieve_player() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let test_name = "test_player1".to_string();
        let mut player = Player::new(&test_name);
        player.set_id(99);
        player.set_money(1000);
        player.set_games(10);
        player.set_wins(8);
        player.set_losses(2);

        let _ = db_client.insert(&player).await.unwrap();

        let collection: Collection<Document> = db_client.database.collection("players");
        let filter = doc! { "player_name": "test_player1" };
        let doc = collection.find_one(filter).await.unwrap();

        let retrieved_player = Player::from_document(&doc.unwrap()).unwrap();
        assert_eq!(retrieved_player.get_id(), player.get_id());
        assert_eq!(retrieved_player.get_name(), player.get_name());
        assert_eq!(retrieved_player.get_money(), player.get_money());
        assert_eq!(retrieved_player.get_games(), player.get_games());
        assert_eq!(retrieved_player.get_wins(), player.get_wins());
        assert_eq!(retrieved_player.get_losses(), player.get_losses());

        db_client.delete_one(&player).await.unwrap();
    }

    #[tokio::test]
    async fn test_dbclient_update_player() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let test_name = "test_player2".to_string();
        let mut player = Player::new(&test_name);
        player.set_id(88);
        player.set_money(1001);
        player.set_games(11);
        player.set_wins(7);
        player.set_losses(4);

        let _inserted_player = db_client.insert(&player).await.unwrap();

        player.set_id(1);
        player.set_money(1);
        player.set_games(1);
        player.set_wins(1);
        player.set_losses(0);

        db_client.update_one(&player).await.unwrap();

        let collection: Collection<Document> = db_client.database.collection("players");
        let filter = doc! { "player_name": "test_player2" };
        let doc = collection.find_one(filter).await.unwrap();

        let updated_player = Player::from_document(&doc.unwrap()).unwrap();
        assert_eq!(updated_player.get_id(), &1);
        assert_eq!(updated_player.get_name(), player.get_name());
        assert_eq!(updated_player.get_money(), &1);
        assert_eq!(updated_player.get_games(), 1);
        assert_eq!(updated_player.get_wins(), 1);
        assert_eq!(updated_player.get_losses(), 0);

        db_client.delete_one(&player).await.unwrap();
    }

    #[tokio::test]
    async fn test_dbclient_find_player() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let test_name = "test_player3".to_string();
        let mut player = Player::new(&test_name);
        player.set_id(11);
        player.set_money(11);
        player.set_games(11);
        player.set_wins(11);
        player.set_losses(11);

        db_client.insert(&player).await.unwrap();

        let queried_player = Player::new(&test_name);

        let result = db_client.query_one(&queried_player).await;

        assert!(result.is_ok());

        match result {
            Ok(Some(retrieved_player)) => {
                assert_eq!(retrieved_player.get_name(), player.get_name());
                assert_eq!(retrieved_player.get_id(), player.get_id());
                assert_eq!(retrieved_player.get_money(), player.get_money());
                assert_eq!(retrieved_player.get_games(), player.get_games());
                assert_eq!(retrieved_player.get_wins(), player.get_wins());
                assert_eq!(retrieved_player.get_losses(), player.get_losses());
            }
            Ok(None) => {
                panic!("Player not found");
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
        db_client.delete_one(&player).await.unwrap();
    }

    #[tokio::test]
    async fn test_dbclient_find_player_not_in_db() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let test_name = "im_not_real".to_string();

        let no_player = Player::new(&test_name);

        let result = db_client.query_one(&no_player).await;

        assert!(result.is_ok());

        match result {
            Ok(Some(_retrieved_player)) => {
                panic!("There shouldnt be a player in the db")
            }
            Ok(None) => {
                println!("All Good!")
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_dbclient_query_all_players() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let player = Player::new("test_player4");
        let player2 = Player::new("test_player5");

        db_client.insert(&player).await.unwrap();
        db_client.insert(&player2).await.unwrap();

        let result = db_client.query_all::<Player>().await.unwrap();

        assert!(result.len() > 0);
        db_client.delete_one(&player).await.unwrap();
        db_client.delete_one(&player2).await.unwrap();
    }

    #[tokio::test]
    async fn test_dbclient_get_new_player_id() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let test_name = "test_player6".to_string();
        let mut player = Player::new(&test_name);
        player.set_id(999);

        let _ = db_client.insert(&player).await.unwrap();

        let next_id = db_client.get_new_player_id().await;

        assert_eq!(next_id, 1000);

        db_client.delete_one(&player).await.unwrap();
    }

    #[tokio::test]
    async fn test_dbclient_get_next_game() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let game_id = 999;
        let game = GameState::empty_constructor(game_id);

        let _ = db_client.insert(&game).await;

        let next_id = db_client.get_new_game_id().await;

        assert_eq!(next_id, 1000);
        db_client.delete_one(&game).await.unwrap();
    }

    #[tokio::test]
    async fn test_dbclient_insert_and_retrieve_game() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let game_id = 998;
        let game_type = "thm".to_string();
        let mut game = GameState::empty_constructor(game_id);
        game.set_variant(&game_type);
        game.set_highest_bet(100);

        let _ = db_client.insert(&game).await.unwrap();

        let queried_game = GameState::empty_constructor(game_id);
        let result = db_client.query_one(&queried_game).await;

        assert!(result.is_ok());

        match result {
            Ok(Some(retrieved_game)) => {
                assert_eq!(retrieved_game.get_game_id(), game.get_game_id());
                //assert_eq!(retrieved_game.get_variant(), game.get_variant());
                assert_eq!(retrieved_game.get_highest_bet(), game.get_highest_bet());
            }
            Ok(None) => {
                panic!("Game not found!");
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }

        db_client.delete_one(&game).await.unwrap();
    }

    #[tokio::test]
    async fn test_dbclient_find_game_not_in_db() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let test_name = 99999;

        let no_game = GameState::empty_constructor(test_name);

        let result = db_client.query_one(&no_game).await;

        assert!(result.is_ok());

        match result {
            Ok(Some(_retrieved_player)) => {
                panic!("There shouldnt be this game in the db")
            }
            Ok(None) => {
                println!("All Good!")
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_dbclient_game_update_game() {
        let db_client = DbClient::new("mongodb://localhost:27017").await.unwrap();

        let game_id = 996;
        let mut game_type = "thm".to_string();
        let mut game = GameState::empty_constructor(game_id);
        game.set_variant(&game_type);
        game.set_highest_bet(100);

        let _ = db_client.insert(&game).await.unwrap();

        game_type = "noholdem".to_string();
        game.set_variant(&game_type);
        game.set_highest_bet(200);

        db_client.update_one(&game).await.unwrap();

        let queried_game = GameState::empty_constructor(game_id);
        let result = db_client.query_one(&queried_game).await;

        assert!(result.is_ok());

        match result {
            Ok(Some(retrieved_game)) => {
                assert_eq!(retrieved_game.get_game_id(), game.get_game_id());
                assert_eq!(retrieved_game.get_variant(), game.get_variant());
                assert_eq!(retrieved_game.get_highest_bet(), game.get_highest_bet());
            }
            Ok(None) => {
                panic!("Game not found!");
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }

        db_client.delete_one(&game).await.unwrap();
    }
}
