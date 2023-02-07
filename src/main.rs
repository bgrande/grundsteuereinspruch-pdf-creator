use std::env;
use std::sync::Arc;
use std::net::SocketAddr;


mod config;
mod crypt;
mod db;
mod form;
mod helper;
mod html;
mod pdf;
mod objects;
mod routes;

use axum::{routing::get, routing::post, Router, ServiceExt};

use dotenv::dotenv;
use tracing::{error, info};

pub(crate) mod send;

use crate::routes::{AppState, create_html, create_pdf, get_html, get_pdf, get_result_page, hello};

//todo: use tower for rate limiting
//use tower_request_id::{RequestIdLayer, RequestId};

#[tokio::main]
async fn main()  {
    dotenv().ok();
    /*let config = GovernorConfigBuilder::default()
    .per_second(4)
    .burst_size(2)
    .finish()
    .unwrap();*/

    let smtp_user = match env::var("SMTP_USER") {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };

    let smtp_pass = match env::var("SMTP_PASS") {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };

    let shared_state = Arc::new(AppState {
        email_user: smtp_user,
        email_pass: smtp_pass,
    });

    let router = Router::new()
        .route("/", get(hello))
        // todo: rate limit this request (only 240 per minute per IP)
        .route("/pdf/:id", post(create_pdf))
        // todo rate limit this request (only 240 per minute per IP)
        .route("/html", post(create_html))
        // todo: rate limit this request (only 10 per minute per IP)
        .route("/pdf/:id/:type", get(get_pdf))
        .route("/page/:id/:type", get(get_html))
        .route("/formresult", get(get_result_page))
        //.route("/page/formresult", get(get_html))
        //.layer(RequestIdLayer)
        .with_state(shared_state)
        ;

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    match axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await {
        Ok(_) => info!("started server successfully"),
        Err(e) => error!("error while starting server: {}", e)
    };
}
