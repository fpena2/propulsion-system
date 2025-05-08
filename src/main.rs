use actix_web::{App, HttpServer, web};
use tokio::sync::watch;
use tracing::error;

mod routes;
mod system;

use system::{CountdownState, MissionComputer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let (tx, rx) = watch::channel(CountdownState::Cancel);

    // Spawn task just for the mission computer
    tokio::spawn(async move {
        if let Err(e) = MissionComputer::new(rx).run().await {
            error!("Mission computer failed: {}", e);
        }
    });

    // Server to listen and send inputs to mission computer
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .service(routes::fire_command)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
