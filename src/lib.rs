use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::ops::Deref;
use std::str::FromStr;

use axum::{routing::get, Router};
use axum::extract::Path;
use sync_wrapper::SyncWrapper;
use html2pdf::{run, CliOptions, Error};
use structopt::StructOpt;

fn html2pdf(path: String) -> Result<(), Error> {
    let options= Vec::from([
        "input", path.as_str()
   ]);
    let opt = CliOptions::from_iter(options);

    // Let's go
    run(opt)
}

fn to_result(from_2pdf: Result<(), Error>) -> &'static str {
    return match from_2pdf.unwrap() {
        () => "done"
    }
}

async fn create_pdf(Path(params): Path<HashMap<String, String>>) -> &'static str {
    let name = params.get("name");
    to_result(html2pdf(format!("{}.html", name.unwrap())))
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let router = Router::new().route("/create/:name", get(create_pdf));
    let sync_wrapper = SyncWrapper::new(router);
    Ok(sync_wrapper)
}