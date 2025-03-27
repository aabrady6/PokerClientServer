use crate::db::dbclient::DbClient;
use crate::game::client_messages;
use crate::game::game_state;
use crate::game::player::Player;
use game::server::{get_game, handle_action, register_player, websocket, Server};
use std::error::Error;
//use ui::menu::{game_stats_menu, main_menu, player_stats_menu, reset_stats_menu, welcome};

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

mod db;
pub mod game;
pub mod game_type;
mod ui;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let uri = "mongodb://localhost:27017";
//     let db_client = DbClient::new(uri).await?;

//     let mut player: Player = welcome(&db_client).await;
//     loop {
//         match db_client.query_one(&player).await {
//             Ok(Some(p)) => player = p,
//             Ok(None) => {
//                 println!("\nPlayer not found.");
//             }
//             Err(e) => {
//                 println!("Error retrieving player: {}", e);
//             }
//         }

//         let user_input = main_menu().await;

//         match user_input.as_str() {
//             "five-card-draw" => {
//                 let mut game = FiveCardDraw::new(LoadState::Empty);

//                 let _ = game.game_state.insert_player(&player.clone());

//                 let vinny = Player::new("fake_vinny");
//                 let sarah = Player::new("fake_sarah");
//                 let wanda = Player::new("fake_wanda");
//                 let player2 = db_client.query_one(&vinny).await;
//                 match player2 {
//                     Ok(Some(retrieved_player)) => {
//                         game.game_state.insert_player(&retrieved_player.clone());
//                     }
//                     Ok(None) => {}
//                     Err(e) => println!("Error: {}", e),
//                 }
//                 let player3 = db_client.query_one(&sarah).await;
//                 match player3 {
//                     Ok(Some(retrieved_player)) => {
//                         game.game_state.insert_player(&retrieved_player.clone());
//                     }
//                     Ok(None) => {}
//                     Err(e) => println!("Error: {}", e),
//                 }
//                 let player4 = db_client.query_one(&wanda).await;
//                 match player4 {
//                     Ok(Some(retrieved_player)) => {
//                         game.game_state.insert_player(&retrieved_player.clone());
//                     }
//                     Ok(None) => {}
//                     Err(e) => println!("Error: {}", e),
//                 }
//                 game.game_run_through().await;

//                 ""
//             }
//             "seven-card-stud" => {
//                 let mut game = SevenCardStud::new(LoadState::Empty);
//                 let _ = game.game_state.insert_player(&player.clone());

//                 let vinny = Player::new("fake_vinny");
//                 let player2 = db_client.query_one(&vinny).await;
//                 match player2 {
//                     Ok(Some(retrieved_player)) => {
//                         let _ = game.game_state.insert_player(&retrieved_player.clone());
//                     }
//                     Ok(None) => {
//                         println!("NO PLAYER IN DB");
//                     }
//                     Err(e) => println!("Error: {}", e),
//                 }

//                 let wanda = Player::new("fake_wanda");
//                 let player4 = db_client.query_one(&wanda).await;
//                 match player4 {
//                     Ok(Some(retrieved_player)) => {
//                         game.game_state.insert_player(&retrieved_player.clone());
//                     }
//                     Ok(None) => {}
//                     Err(e) => println!("Error: {}", e),
//                 }

//                 game.game_run_through().await;
//                 ""
//             }
//             "texas-hold-em" => {
//                 let mut game = TexasHoldEm::new(LoadState::Empty);
//                 let _ = game.game_state.insert_player(&player.clone());

//                 let vinny = Player::new("fake_vinny");
//                 let player2 = db_client.query_one(&vinny).await;
//                 match player2 {
//                     Ok(Some(retrieved_player)) => {
//                         let _ = game.game_state.insert_player(&retrieved_player.clone());
//                     }
//                     Ok(None) => {
//                         println!("NO PLAYER IN DB");
//                     }
//                     Err(e) => println!("Error: {}", e),
//                 }

//                 let wanda = Player::new("fake_wanda");
//                 let player4 = db_client.query_one(&wanda).await;
//                 match player4 {
//                     Ok(Some(retrieved_player)) => {
//                         game.game_state.insert_player(&retrieved_player.clone());
//                     }
//                     Ok(None) => {}
//                     Err(e) => println!("Error: {}", e),
//                 }

//                 game.game_run_through().await;
//                 ""
//             }
//             "player-stats" => {
//                 player_stats_menu(&db_client).await;
//                 ""
//             }
//             "game-stats" => {
//                 game_stats_menu(&db_client).await;
//                 ""
//             }
//             "reset-stats" => {
//                 reset_stats_menu(&db_client, &mut player).await;
//                 ""
//             }
//             "exit" => break,
//             _ => "",
//         };
//     }

//     Ok(())
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = Server::new(); // Returns Arc<Server<T>>
    let server_data = web::Data::from(server.clone()); // ✅ Store Arc<Server<T>> directly
    let server_clone = server.clone();

    println!("Starting broadcast listener...");
    // Start broadcast listener
    tokio::spawn(async move {
        server_clone.listen_for_broadcasts().await;
    });

    println!("Rust backend listening on http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone()) // ✅ Pass Arc<Server<T>> correctly
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(get_game)
            .service(register_player)
            .service(handle_action)
            .service(websocket)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
