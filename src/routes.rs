use actix_web::{HttpResponse, Responder, post, web};
use tokio::sync::watch;

use crate::system::CountdownCommand;

#[derive(serde::Deserialize)]
struct FireCommand {
    countdown: String,
}

#[post("/")]
pub async fn fire_command(
    data: web::Json<FireCommand>,
    tx: web::Data<watch::Sender<CountdownCommand>>,
) -> impl Responder {
    let command = data.countdown.as_str();
    let command = match data.countdown.as_str() {
        "-1" => CountdownCommand::Cancel,
        "reset" => CountdownCommand::Reset,
        _ => match command.parse::<u64>() {
            Ok(value) => CountdownCommand::Start(value as u64),
            Err(_) => CountdownCommand::Invalid,
        },
    };

    if command == CountdownCommand::Invalid {
        return HttpResponse::InternalServerError().body("Error");
    }

    match tx.send(command) {
        Ok(_) => HttpResponse::Ok().body("Success"),
        Err(_) => HttpResponse::InternalServerError().body("Error"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test, web};
    use tokio::sync::watch;

    #[actix_web::test]
    async fn test_fire_command_start() {
        let (tx, _rx) = watch::channel(CountdownCommand::Cancel);
        let tx = web::Data::new(tx);
        let app = test::init_service(App::new().app_data(tx.clone()).service(fire_command)).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({"countdown": "5"}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_fire_command_cancel() {
        let (tx, _rx) = watch::channel(CountdownCommand::Cancel);
        let tx = web::Data::new(tx);
        let app = test::init_service(App::new().app_data(tx.clone()).service(fire_command)).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({"countdown": "-1"}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_fire_command_invalid() {
        let (tx, _rx) = watch::channel(CountdownCommand::Cancel);
        let tx = web::Data::new(tx);
        let app = test::init_service(App::new().app_data(tx.clone()).service(fire_command)).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({"countdown": "-2"}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);
    }
}
