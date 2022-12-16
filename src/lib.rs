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
    let name = params.get("id");
    // todo create pdf by given id
    // todo 0. rate limit this request (only 240 per minute per IP)
    // todo 1. create another random hash? Or use the given one
    // todo 2. get html files by html/{file_name}/{grundsteuereinspruch|invoice}.html -> convert to pdf/{file_name}/{id}.pdf + pdf/{file_name}/{id}_invoice.pdf
    // todo 3. send invoice link to me via email
    // todo 4. send overview link to me via email
    to_result(html2pdf(format!("{}.html", name.unwrap())))
}

async fn create_html(Path(params): Path<HashMap<String, String>>) -> &'static str {
    // todo: 0. rate limit this request (only 240 per minute per IP)
    // todo: 1. create file_name: (true random hash)
    // todo: 2. process json body, also check for API token
    // todo: 3. create html file for html/{file_name}/grundsteuereinspruch.html (tera)
    // todo: 4. create html file for html/{file_name}/rechnung.html             (tera)
    // todo: 5. create html file for html/{file_name}/index.html (contains links to letter + rechnung (pdf): /pdf/get/{file_name})
    // todo: 6. trigger create_pdf for {file_name}
    let id = params.get("id");
    to_result(html2pdf(format!("{}.html", id.unwrap())))
}

async fn hello() -> &'static str {
    "v1"
}

async fn get_pdf(Path(params): Path<HashMap<String, String>>) -> &'static str {
    // todo: get PDF by id
    // todo: 1. rate limit this request (only 10 per minute per IP)
    // todo: 2. show pdf file by id, show error if not existing

    "v1"
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello))
        .route("/pdf/create/:id", post(create_pdf))
        .route("/html/create", post(create_html))
        .route("/pdf/get/:id", post(get_pdf))
        ;
    let sync_wrapper = SyncWrapper::new(router);
    Ok(sync_wrapper)
}