use actix_web::{get, post, HttpResponse, HttpServer, Responder};

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        body::{BodySize, MessageBody},
        test, web, App,
    };

    const HELLO_MSG: &str = "Hello world!";

    #[get("/")]
    async fn hello() -> impl Responder {
        HttpResponse::Ok().body(HELLO_MSG)
    }

    #[post("/echo")]
    async fn echo(req_body: String) -> impl Responder {
        HttpResponse::Ok().body(req_body)
    }

    async fn manual_hello() -> impl Responder {
        HttpResponse::Ok().body("Hey there!")
    }

    #[ignore]
    #[actix_web::test]
    async fn test_server() -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
                .service(hello)
                .service(echo)
                .route("/hey", web::get().to(manual_hello))
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
    }

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new().service(hello)).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_and_read_body(&app, req).await;
        assert_eq!(resp, HELLO_MSG.as_bytes());

        let req = test::TestRequest::get().uri("/does-not-exist").to_request();
        let resp = test::call_and_read_body(&app, req).await;
        assert!(resp.size() == BodySize::ZERO);
    }
}
