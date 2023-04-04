use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    guard, middleware::Logger, web, App, HttpResponse, HttpResponseBuilder, HttpServer,
};
use clap::Parser;
use etsi_mec_qkd::applicationlistserver::{build_application_list_server, ApplicationListServer};
use etsi_mec_qkd::lcmpserver::LcmpServer;
use etsi_mec_qkd::messages::{AppContext, ApplicationListInfo, ProblemDetails, Validate};
use log::info;
use serde::__private::de::Content;
use std::sync::Mutex;

/// Return an HTTP response with a Problem Details body
fn problem_details_response(status_code: StatusCode, error: &str) -> HttpResponse {
    let p = ProblemDetails {
        status: status_code.as_u16().into(),
        detail: error.to_string(),
    };
    HttpResponseBuilder::new(status_code)
        .insert_header(ContentType::json())
        .body(serde_json::to_string(&p).unwrap_or_default())
}

/// Return an HTTP OK response
fn ok_response<T: serde::Serialize>(body: &T) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(ContentType::json())
        .body(serde_json::to_string(&body).unwrap_or_default())
}

/// Command-line arguments
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
    #[arg(long, default_value_t = String::from("single;10,URI"))]
    app_context_type: String,
}

/// An ETSI MEC Life Cycle Management Proxy
struct AppState {
    lcmp_server: Mutex<LcmpServer>,
}

/// Handler for GET /app_list
async fn app_list(
    info: web::Query<ApplicationListInfo>,
    data: web::Data<AppState>,
) -> HttpResponse {
    match info.validate() {
        Err(err) => problem_details_response(StatusCode::BAD_REQUEST, err.as_str()),
        Ok(_) => match data
            .lcmp_server
            .lock()
            .unwrap()
            .application_list()
            .application_list(info.0)
        {
            Ok(x) => ok_response(&x),
            Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
        },
    }
}

/// Handler for POST /app_contexts
async fn app_contexts(data: web::Data<AppState>, body: String) -> HttpResponse {
    let mut x: Result<AppContext, serde_json::Error> = serde_json::from_str(&body);
    match &mut x {
        Ok(app_context) => {
            match data
                .lcmp_server
                .lock()
                .unwrap()
                .app_context()
                .new_context(app_context)
            {
                Ok(_) => ok_response(&app_context),
                Err(err) => problem_details_response(StatusCode::FORBIDDEN, err.as_str()),
            }
        }
        Err(err) => problem_details_response(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    }
}

/// Handler for DELETE /app_contexts/{contextId}
async fn delete_context(data: web::Data<AppState>, info: web::Path<String>) -> HttpResponse {
    match data
        .lcmp_server
        .lock()
        .unwrap()
        .app_context()
        .del_context(&info)
    {
        Ok(_) => HttpResponse::NoContent().into(),
        Err(err) => problem_details_response(StatusCode::NOT_FOUND, err.as_str()),
    }
}

/// Handler for UPDATE /app_contexts/{contextId}
async fn update_context(
    data: web::Data<AppState>,
    body: String,
    info: web::Path<String>,
) -> HttpResponse {
    let mut x: Result<AppContext, serde_json::Error> = serde_json::from_str(&body);
    match &mut x {
        Ok(app_context) => {
            if let Some(context_id) = &app_context.contextId {
                if context_id != info.as_str() {
                    return problem_details_response(
                        StatusCode::BAD_REQUEST,
                        "context ID in the request does not match the path",
                    );
                }
            }
            match data
                .lcmp_server
                .lock()
                .unwrap()
                .app_context()
                .update_context(app_context)
            {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(err) => problem_details_response(StatusCode::FORBIDDEN, err.as_str()),
            }
        }
        Err(err) => problem_details_response(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    }
}

/// Handler for GET /app_contexts/{contextId}
/// This method is *not* ETSI MEC standard
async fn get_context(data: web::Data<AppState>, info: web::Path<String>) -> HttpResponse {
    match data
        .lcmp_server
        .lock()
        .unwrap()
        .app_context()
        .get_context(&info)
    {
        Ok(app_context) => ok_response(&app_context),
        Err(err) => problem_details_response(StatusCode::NOT_FOUND, err.as_str()),
    }
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
            .service(
                web::resource("/dev_app/v1/app_contexts/{contextId}")
                    .guard(guard::Header("content-type", "application/json"))
                    .route(web::put().to(update_context)),
            )
            .service(
                web::resource("/dev_app/v1/app_contexts/{contextId}")
                    .route(web::delete().to(delete_context))
                    .route(web::get().to(get_context)),
            )
    })
    .bind(args.address)?
    .workers(args.workers)
    .run()
    .await
}
