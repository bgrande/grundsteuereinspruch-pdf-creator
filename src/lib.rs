use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::ops::Deref;
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

fn to_result(from_2pdf: Result<(), Error>) -> &'static str {
    return match from_2pdf.unwrap() {
        () => "done",
        _ => "sth. went wrong"
    }
}

async fn hello_world() -> &'static str {
    to_result(html2pdf("https://bgrande.de".to_string()))
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let router = Router::new().route("/hello", get(hello_world));
    let sync_wrapper = SyncWrapper::new(router);
    Ok(sync_wrapper)
}