use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use game::server::{player_login, player_register, stats, websocket, startgame, Server};

mod db;
pub mod game;
mod ui;

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

    println!("Rust backend listening on http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone()) // ✅ Pass Arc<Server<T>> correctly
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(player_login)
            .service(player_register)
            .service(websocket)
            .service(stats)
            .service(startgame)
        })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
