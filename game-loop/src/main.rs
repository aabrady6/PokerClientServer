use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use game::server::{
    admin_controls, player_login, player_register, startgame, stats, websocket, Server,
};

mod db;
pub mod game;
mod ui;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = Server::new();
    let server_data = web::Data::from(server.clone());
    let server_clone = server.clone();

    println!("Starting broadcast listener...");
    tokio::spawn(async move {
        server_clone.listen_for_broadcasts().await;
    });
    println!("Rust backend listening on http://0.0.0.0:8080");

    tokio::spawn(async move {
        admin_controls().await;
    });
    HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
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
