use actix_web::http::header::ContentType;
use actix_web::{guard, middleware::Logger, web, App, HttpResponse, HttpServer};
use clap::Parser;
use etsi_mec_qkd::applicationlistserver::{build_application_list_server, ApplicationListServer};
use etsi_mec_qkd::lcmpserver::LcmpServer;
use etsi_mec_qkd::messages::{AppContext, ApplicationListInfo, ProblemDetails, Validate};
use log::info;
use serde::__private::de::Content;
use std::sync::Mutex;

/// Return a 400 Bad Request with Problem Details filled with passed argument.
fn bad_request(error: &str) -> HttpResponse {
    let p = ProblemDetails {
        status: 400,
        detail: error.to_string(),
    };
    HttpResponse::BadRequest()
        .insert_header(ContentType::json())
        .body(serde_json::to_string(&p).unwrap_or_default())
}

/// An ETSI MEC Life Cycle Management Proxy
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

    /// Application context manager type
    #[arg(long, default_value_t = String::from("static;file=app_context.json"))]
    app_context_type: String,
}

struct AppState {
    lcmp_server: Mutex<LcmpServer>,
}

async fn app_list(
    info: web::Query<ApplicationListInfo>,
    data: web::Data<AppState>,
) -> HttpResponse {
    match info.validate() {
        Err(err) => bad_request(err.as_str()),
        Ok(_) => match data
            .lcmp_server
            .lock()
            .unwrap()
            .application_list()
            .application_list(info.0)
        {
            Ok(x) => HttpResponse::Ok()
                .insert_header(ContentType::json())
                .body(serde_json::to_string(&x).unwrap_or_default()),
            Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
        },
    }
}

async fn app_contexts(data: web::Data<AppState>, body: String) -> HttpResponse {
    let x: Result<AppContext, serde_json::Error> = serde_json::from_str(&body);
    match x {
        Ok(app_context) => {
            HttpResponse::Ok().body(serde_json::to_string(&app_context).unwrap_or_default())
        }
        Err(err) => bad_request(err.to_string().as_str()),
    }

    // match data
    //     .lcmp_server
    //     .lock()
    //     .unwrap()
    //     .application_list()
    //     .application_list(info.0)
    // {
    //     Ok(x) => HttpResponse::Ok()
    //         .insert_header(ContentType::json())
    //         .body(serde_json::to_string(&x).unwrap_or_default()),
    //     Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
    // }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let state = web::Data::new(AppState {
        lcmp_server: Mutex::new(
            LcmpServer::build(&args.app_list_type, &args.app_context_type)
                .expect("could not create the LCMP server"),
        ),
    });

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!(
        "starting HTTP server with {} workers at {}",
        args.workers, args.address
    );
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(
                web::resource("/dev_app/v1/app_list")
                    .guard(guard::Header("content-type", "application/json"))
                    .route(web::get().to(app_list)),
            )
            .service(
                web::resource("/dev_app/v1/app_contexts")
                    .guard(guard::Header("content-type", "application/json"))
                    .route(web::post().to(app_contexts)),
            )
    })
    .bind(args.address)?
    .workers(args.workers)
    .run()
    .await
}
