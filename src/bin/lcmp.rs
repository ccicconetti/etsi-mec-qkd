use actix_web::http::header::ContentType;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use clap::Parser;
use etsi_mec_qkd::stateserver::{build_application_list_server, ApplicationListServer};
use serde::Serialize;
use std::sync::Mutex;

/// A ETSI MEC Life Cycle Management Proxy
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address and port of the HTTP server
    #[arg(long, default_value_t = String::from("0.0.0.0:8080"))]
    address: String,

    /// Number of parallel workers
    #[arg(short, long, default_value_t = 1)]
    workers: usize,

    /// Application list manager type
    #[arg(long, default_value_t = String::from("static;file=application_list.json"))]
    app_list_type: String,
}

struct AppState {
    app_list_server: Mutex<Box<dyn ApplicationListServer + Send + Sync>>,
}

#[get("/dev_app/v1/app_list")]
async fn app_list(data: web::Data<AppState>) -> impl Responder {
    match data.app_list_server.lock().unwrap().application_list() {
        Ok(x) => HttpResponse::Ok()
            .insert_header(ContentType::json())
            .body(serde_json::to_string(&x).unwrap_or_default()),
        Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let state = web::Data::new(AppState {
        app_list_server: Mutex::new(
            build_application_list_server(&args.app_list_type)
                .expect("could not create the ApplicationList server"),
        ),
    });

    println!(
        "starting HTTP server with {} workers at {}",
        args.workers, args.address
    );
    HttpServer::new(move || App::new().app_data(state.clone()).service(app_list))
        .bind(args.address)?
        .workers(args.workers)
        .run()
        .await
}
