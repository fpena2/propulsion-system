use actix_web::{HttpResponse, Responder, post, web};
use tokio::sync::watch;

use crate::system::CountdownState;

#[derive(serde::Deserialize)]
struct FireCommand {
    countdown: i64,
}

#[post("/")]
pub async fn fire_command(
    data: web::Json<FireCommand>,
    tx: web::Data<watch::Sender<CountdownState>>,
) -> impl Responder {
    let command = match data.countdown {
        -1 => CountdownState::Cancel,
        n if n >= 0 => CountdownState::Start(n as u64),
        _ => return HttpResponse::BadRequest().body("Invalid countdown"),
    };

    tx.send(command).expect("Failed to send fire command");

    HttpResponse::Ok().body("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test, web};
    use tokio::sync::watch;

    #[actix_web::test]
    async fn test_fire_command_start() {
        let (tx, _rx) = watch::channel(CountdownState::Cancel);
        let tx = web::Data::new(tx);
        let app = test::init_service(App::new().app_data(tx.clone()).service(fire_command)).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({"countdown": 5}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_fire_command_cancel() {
        let (tx, _rx) = watch::channel(CountdownState::Cancel);
        let tx = web::Data::new(tx);
        let app = test::init_service(App::new().app_data(tx.clone()).service(fire_command)).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({"countdown": -1}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_fire_command_invalid() {
        let (tx, _rx) = watch::channel(CountdownState::Cancel);
        let tx = web::Data::new(tx);
        let app = test::init_service(App::new().app_data(tx.clone()).service(fire_command)).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({"countdown": -2}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }
}
