use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::str::FromStr;

use axum::{routing::get, Router};
use sync_wrapper::SyncWrapper;
use html2pdf::{run, CliOptions, Error};
use structopt::StructOpt;

fn html2pdf(url: String) -> Result<(), Error> {
    let options= Vec::from([
        "input".to_string(), url,
        "output".to_string(), "test.pdf".to_string(),
   ]);
    let opt = CliOptions::from_iter(options);

    // Let's go
    run(opt)
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let router = Router::new().route("/hello", get(hello_world));
    let sync_wrapper = SyncWrapper::new(router);
    html2pdf("https://bgrande.de".to_string());
    Ok(sync_wrapper)
}