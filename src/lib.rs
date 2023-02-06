use std::sync::Arc;

mod config;
mod crypt;
mod db;
mod form;
mod helper;
mod html;
mod pdf;
mod objects;
mod routes;

use axum::{routing::get, routing::post, Router};
use sync_wrapper::SyncWrapper;
use shuttle_secrets::SecretStore;

use dotenv::dotenv;
pub(crate) mod send;

use crate::routes::{AppState, create_html, create_pdf, get_html, get_pdf, get_result_page, hello};

//todo: use tower for rate limiting
//use tower_request_id::{RequestIdLayer, RequestId};

fn get_secret(secret_store: &SecretStore, name: &str) -> String {
    return if let Some(secret) = secret_store.get(name) {
        secret
    } else {
        "".to_string()
    };
}

#[shuttle_service::main]
async fn axum(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_service::ShuttleAxum {
    dotenv().ok();
    /*let config = GovernorConfigBuilder::default()
    .per_second(4)
    .burst_size(2)
    .finish()
    .unwrap();*/

    let shared_state = Arc::new(AppState {
        email_user: get_secret(&secret_store, "SMTP_USER"),
        email_pass: get_secret(&secret_store, "SMTP_PASS"),
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
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
