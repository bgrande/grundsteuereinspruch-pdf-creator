use std::collections::HashMap;
use std::{env, fs};

use anyhow::Result as AnyResult;
use axum::extract::Path;
use axum::{extract, routing::get, routing::post, Router};
use chrono::{NaiveDateTime, Utc};
use html2pdf::{run, CliOptions, Error as H2PError};
use sync_wrapper::SyncWrapper;

use rand::{Rng};
use ring::digest::{Context as RingContext, Digest, SHA256};
use serde::{Deserialize, Serialize};

use structopt::lazy_static::lazy_static;
use structopt::StructOpt;

use tera::{Context, Tera};

//todo: use tower for rate limiting
const APP_TOKEN: &str = "846uoisdhgsdgszdog7846934634089hhuaip12xbo";

const TEMPLATE_PATH: &str = "/data/templates";
const TARGET_PATH: &str = "/data/diedaten";

const TEMPLATE_NAME_INDEX: &str = "index.html";
const TEMPLATE_NAME_LETTER: &str = "letter.html";
const TEMPLATE_NAME_INVOICE: &str = "invoice.html";
const TEMPLATE_NAME_LIST: &str = "list.html";

const RESULT_NAME_JSON: &str = "{}.json";

const RESULT_NAME_LETTER: &str = "Grundsteuereinspruch.pdf";
const RESULT_NAME_INVOICE: &str = "Grundsteuereinspruch-Rechnung.pdf";
const RESULT_NAME_LIST: &str = "Grundsteuereinspruch-Liste.pdf";

const FORM_ID: &str = "wvXAdv";
const FORM_NAME: &str = "Einspruch Grundsteuerbescheid";

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let current_dir = String::from(env::current_dir().unwrap().to_str().unwrap());
        let template_path = format!(
            "{}/{}/*.html",
            current_dir,
            TEMPLATE_PATH
        );

        let mut tera = match Tera::new(template_path.as_str()) {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        //tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

fn html2pdf(path: String) -> std::result::Result<(), H2PError> {
    let options = Vec::from(["input", path.as_str()]);
    let opt = CliOptions::from_iter(options);

    // Let's go
    run(opt)
}

fn to_result(from_2pdf: ()) -> &'static str {
    return match from_2pdf {
        () => "done",
    };
}

// todo check for before processing:
// const FORM_ID: &str = "wvXAdv"
// const ORIGIN_PAGE: &str = "/fragebogen.html"
// const TOKEN_ID: &str = "???TOCREATE" -> add as param

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
struct FormFieldOption {
    id: String,
    text: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
struct FormField {
    key: String,
    label: String,
    #[serde(rename = "type")]
    form_type: String,
    value: Vec<String>,
    options: Option<Vec<FormFieldOption>>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
struct QuestionData {
    response_id: String,
    submission_id: String,
    respondent_id: String,
    form_id: String,
    form_name: String,
    created_at: String,
    fields: Vec<FormField>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
struct QuestionResult {
    event_id: String,
    event_type: String,
    created_at: String,
    data: QuestionData,
}

struct FormFieldMeta {
    key: String, /// ref, originPage, token
    value: String,
}

#[derive(Serialize)]
struct Letter {
    first_name: String,
    last_name: String,
    street: String,
    number: String,
    zip: String,
    city: String,
    email: String,
    phone: String,
    max_sender_count: i8,
    reference_number: String,
    sender_names_intro: String,
    sender_names: Vec<String>,
    receiver_office_name: String,
    receiver_office_address: String,
    receiver_office_zip: String,
    receiver_office_city: String,
    objection_subjects: Vec<String>,
    objection_subject_start_dates: Vec<String>,
    objection_subject_reasons: Vec<String>,
    date_created: String,
    subject_text: String,
    main_text: String,
    additional_greeting_text: String,
}
#[derive(Serialize)]
struct Payment {
    price: String,
    currency: String,
    name: String,
    email: String,
    link: String,
}
#[derive(Serialize)]
struct Invoice {
    sender_name_first: String,
    sender_name_last: String,
    sender_company_name: String,
    sender_street: String,
    sender_number: String,
    sender_zip: String,
    sender_city: String,
    first_name: String,
    last_name: String,
    street: String,
    number: String,
    zip: String,
    city: String,
    email: String,
    date: String,
    invoice_id: String,
    customer_id: String,
    subject_text: String,
    payment: Payment,
    invoice_objects: Vec<String>,
}
#[derive(Serialize)]
struct Index {
    first_name: String,
    last_name: String,
    date: String,
    file_id: String,
    deadline_date: String,
}
#[derive(Serialize)]
struct List {
    first_name: String,
    last_name: String,
    date: String,
    file_id: String,
    deadline_date: String,
}
#[derive(Serialize)]
struct EMail {
    first_name: String,
    last_name: String,
    email: String,
    date: String,
    link: String,
    deadline_date: String,
}

fn create_random_id() -> AnyResult<Digest> {
    let mut random_gen = rand::thread_rng();
    let mut context = RingContext::new(&SHA256);

    // add a lot of random numbers
    for _i in 1..250 {
        let data = random_gen.gen();
        context.update(&[data]);
    }

    Ok(context.finish())
}

fn get_target_path(id: String) -> AnyResult<String> {
    let current_dir = String::from(env::current_dir()?.to_str().unwrap());
    let html_path = format!("{}/{}/{}", current_dir, TARGET_PATH, id);

    Ok(html_path)
}

fn create_path(creator_id: AnyResult<Digest>) -> AnyResult<String> {
    let create_id_folder = data_encoding::HEXLOWER.encode(creator_id?.as_ref());
    let path = get_target_path(create_id_folder)?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn is_valid_payload(payload: &QuestionResult) -> bool {
    let sent = NaiveDateTime::parse_from_str(
        payload.created_at.replace("Z", "+00:00").as_str(),
        "%Y-%m-%dT%H:%M:%S%z",
    );
    let sent_date_time = match sent {
        Ok(date_time) => date_time.timestamp(),
        Err(_) => return false,
    };

    let now = Utc::now().timestamp();
    payload.event_type == "FORM_RESPONSE"
        && sent_date_time <= now.to_owned()
        && payload.event_id.contains("-")
        && payload.data.response_id != ""
        && payload.data.submission_id != ""
        && payload.data.response_id != ""
        && payload.data.form_id == FORM_ID
        && payload.data.form_name == FORM_NAME
}

fn create_pdf_by_id(creator_id: String) {}

async fn create_pdf(Path(params): Path<HashMap<String, String>>) -> &'static str {
    let name = match params.get("id") {
        Some(id) => id,
        _ => "",
    };

    // todo create pdf by given id
    // todo 1. create another random hash? Or use the given one
    // todo 2. get html files by html/{file_name}/{grundsteuereinspruch|invoice}.html -> convert to pdf/{file_name}/{id}.pdf + pdf/{file_name}/{id}_invoice.pdf
    // todo 3. send invoice link to me via email
    // todo 4. send overview link to me via email
    let html2pdf = match html2pdf(format!("data/templates/{}.html", name)) {
        Ok(result) => result,
        Err(e) => return "sth. went wrong",
    };

    to_result(html2pdf)
}

async fn create_html(
    Path(params): Path<HashMap<String, String>>,
    extract::Json(payload): extract::Json<QuestionResult>,
) -> &'static str {
    let is_payload_valid = is_valid_payload(&payload);

    if !is_payload_valid {
        return "The payload sent is invalid";
    }

    let creator_id = create_random_id();

    let mut letter = Letter {
        first_name: "".to_string(),
        last_name: "".to_string(),
        street: "".to_string(),
        number: "".to_string(),
        zip: "".to_string(),
        city: "".to_string(),
        email: "".to_string(),
        phone: "".to_string(),
        max_sender_count: 0,
        reference_number: "".to_string(),
        sender_names_intro: "".to_string(),
        sender_names: vec![],
        receiver_office_name: "".to_string(),
        receiver_office_address: "".to_string(),
        receiver_office_zip: "".to_string(),
        receiver_office_city: "".to_string(),
        objection_subjects: vec![],
        objection_subject_start_dates: vec![],
        objection_subject_reasons: vec![],
        date_created: "".to_string(),
        subject_text: "".to_string(),
        main_text: "".to_string(),
        additional_greeting_text: "".to_string(),
    };
    let mut invoice = Invoice {
        sender_name_first: "".to_string(),
        sender_name_last: "".to_string(),
        sender_company_name: "".to_string(),
        sender_street: "".to_string(),
        sender_number: "".to_string(),
        sender_zip: "".to_string(),
        sender_city: "".to_string(),
        first_name: "".to_string(),
        last_name: "".to_string(),
        street: "".to_string(),
        number: "".to_string(),
        zip: "".to_string(),
        city: "".to_string(),
        email: "".to_string(),
        date: "".to_string(),
        invoice_id: "".to_string(),
        customer_id: "".to_string(),
        subject_text: "".to_string(),
        payment: Payment {
            price: "".to_string(),
            currency: "".to_string(),
            name: "".to_string(),
            email: "".to_string(),
            link: "".to_string(),
        },
        invoice_objects: vec![],
    };
    let mut index = Index {
        first_name: "".to_string(),
        last_name: "".to_string(),
        date: "".to_string(),
        file_id: "".to_string(),
        deadline_date: "".to_string(),
    };
    let mut list = List {
        first_name: "".to_string(),
        last_name: "".to_string(),
        date: "".to_string(),
        file_id: "".to_string(),
        deadline_date: "".to_string(),
    };
    let mut email = EMail {
        first_name: "".to_string(),
        last_name: "".to_string(),
        email: "".to_string(),
        date: "".to_string(),
        link: "".to_string(),
        deadline_date: "".to_string(),
    };

    let mut meta_token: FormFieldMeta = FormFieldMeta {
        key: "".to_string(),
        value: "".to_string(),
    };
    let mut meta_reference: FormFieldMeta = FormFieldMeta {
        key: "".to_string(),
        value: "".to_string(),
    };
    let mut meta_origin_page: FormFieldMeta = FormFieldMeta {
        key: "".to_string(),
        value: "".to_string(),
    };

    let mut meta_start_now: FormFieldMeta = FormFieldMeta {
        key: "".to_string(),
        value: "".to_string(),
    };
    let mut meta_no_revocation: FormFieldMeta = FormFieldMeta {
        key: "".to_string(),
        value: "".to_string(),
    };

    // todo write the whole payload into JSON?

    for field in payload.data.fields {
        // meta_ref, meta_origin are for analytics, origin and token for another API check as well
        if field.key == "question_3xzMoo_81f3e592-de5c-48f2-b459-b494d65dfc65" {
            meta_reference = FormFieldMeta {
                key: field.key.clone(),
                value: field.value[0].clone(),
            };
        }

        if field.key == "question_3xzMoo_60b2c1c7-03d6-43ae-9266-fd5c29143450" {
            meta_origin_page = FormFieldMeta {
                key: field.key.clone(),
                value: field.value[0].clone(),
            };
        }

        if field.key == "question_3xzMoo_80705223-756a-4a8d-aa1e-e8b0147a977c" {
            meta_token = FormFieldMeta {
                key: field.key.clone(),
                value: field.value[0].clone(),
            };
        }

        if field.key == "question_wzdyR1" {
            letter.first_name = field.value[0].clone();
            index.first_name = field.value[0].clone();
            list.first_name = field.value[0].clone();
            invoice.first_name = field.value[0].clone();
            email.first_name = field.value[0].clone();
        }

        if field.key == "question_m6p5BP" {
            letter.last_name = field.value[0].clone();
            index.last_name = field.value[0].clone();
            list.last_name = field.value[0].clone();
            invoice.last_name = field.value[0].clone();
            email.last_name = field.value[0].clone();
        }

        if field.key == "question_w7p0e6" {
            letter.street = field.value[0].clone();
            invoice.street = field.value[0].clone();
        }

        if field.key == "question_mV47y6" {
            letter.number = field.value[0].clone();
            invoice.number = field.value[0].clone();
        }

        if field.key == "question_nPGQyx" {
            letter.zip = field.value[0].clone();
            invoice.zip = field.value[0].clone();
        }

        if field.key == "question_3ENVY2" {
            letter.city = field.value[0].clone();
            invoice.city = field.value[0].clone();
        }

        if field.key == "question_wazO6q" {
            letter.email = field.value[0].clone();
            invoice.email = field.value[0].clone();
            email.email = field.value[0].clone();
        }

        if field.key == "question_w2RJMg" {
            letter.phone = field.value[0].clone();
        }

        if field.key == "question_nGQE8p" {
            letter.sender_names.push(field.value[0].clone());
        }

        if field.key == "question_wAjR0o" {
            letter.max_sender_count = match field.value[0].clone().parse() {
                Ok(number) => number,
                Err(_) => 0,
            };
        }

        if field.key == "question_nGQE8p" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_mV472J" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_waaJ7v" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_m6PD7B" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_nPrDxB" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_nrYB42" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_3jzbRJ" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_3xzjqv" {
            letter.sender_names.push(field.value[0].clone());
        }
        if field.key == "question_3NPDjO" {
            letter.sender_names.push(field.value[0].clone());
        }

        if field.key == "question_mJWZ0r" {
            index.deadline_date = field.value[0].clone();
            list.deadline_date = field.value[0].clone();
            email.deadline_date = field.value[0].clone();
        }

        if field.key == "question_wgkx4P" {
            letter.reference_number = field.value[0].clone();
        }

        if field.key == "question_nrk9WX" {
            letter.receiver_office_zip = field.value[0].clone();
            // todo get the receiver address by the zip
        }

        // this is Einspruch für Grundsteuerwertbescheid
        if field.key == "question_nWjbDJ_11f917de-4e6a-4290-838e-9d194afd11af"
            && field.value[0].parse() == Ok(true)
        {
            letter
                .objection_subjects
                .push("Grundsteuerwertbescheid".to_string());
        }
        // this is Einspruch für Grundsteuermessbescheid
        if field.key == "question_nWjbDJ_73a81f7f-2dbe-4fcd-9573-38f98596049f"
            && field.value[0].parse() == Ok(true)
        {
            letter
                .objection_subjects
                .push("Grundsteuermessbescheid".to_string());
        }
        // this is Einspruch für Grundsteuerwertbescheid
        if field.key == "question_w8Rgel" && !field.value.is_empty() {
            letter
                .objection_subject_start_dates
                .push(field.value[0].clone());
        }
        // this is Einspruch für Grundsteuermessbescheid
        if field.key == "question_n0DgP9" && !field.value.is_empty() {
            letter
                .objection_subject_start_dates
                .push(field.value[0].clone());
        }

        if field.key == "question_mRjGNv" && !field.value.is_empty() {
            let options = match field.options {
                Some(option) => option,
                None => vec![],
            };

            for value in field.value.clone() {
                match options.iter().find(|&item| item.id == value) {
                    Some(option) => letter.objection_subject_reasons.push(option.text.clone()),
                    None => (),
                };
            }
        }

        if field.key == "question_npkqkZ" && !field.value.is_empty() {
            letter
                .objection_subject_reasons
                .push(field.value[0].clone());
        }

        if field.key == "question_npkqkZ" && !field.value.is_empty() {
            letter
                .objection_subject_reasons
                .push(field.value[0].clone());
        }

        if field.key == "question_3ENVAL_price" {
            invoice.payment.price = field.value[0].clone();
        }
        if field.key == "question_3ENVAL_currency" {
            invoice.payment.currency = field.value[0].clone();
        }
        if field.key == "question_3ENVAL_name" {
            invoice.payment.name = field.value[0].clone();
        }
        if field.key == "question_3ENVAL_email" {
            invoice.payment.email = field.value[0].clone();
        }
        if field.key == "question_3ENVAL_link" {
            invoice.payment.link = field.value[0].clone();
        }

        if field.key == "question_nrYOOl_7200bf88-f16a-4384-9d85-06c7b97b6a4a" {
            meta_start_now = FormFieldMeta {
                key: field.key.clone(),
                value: field.value[0].clone(),
            };
        }
        if field.key == "question_w4O77k_ba1c873b-1bbc-4b00-94a4-0cec5d3b3655" {
            meta_no_revocation = FormFieldMeta {
                key: field.key.clone(),
                value: field.value[0].clone(),
            };
        }
    }

    // todo log meta

    if meta_start_now.value.parse() == Ok(false) || meta_no_revocation.value.parse() == Ok(false) {
        return "Die Zustimmung zur Ausführung des Vertrags vor Ablauf der Widerrufsfrist und/oder den Verlust des Widerrufsrechts dadurch fehlt.";
    }
    if meta_origin_page.value != "/fragebogen.html" || meta_token.value != APP_TOKEN {
        return "Der Aufruf war fehlerhaft!";
    }

    let path = match create_path(creator_id) {
        Ok(path) => path,
        Err(e) => {
            //let error_msg = e.to_string().as_str();
            return "Etwas lief schief beim Erstellen des Briefs (1).";
            // todo add kontakt-link
        }
    };

    let index_path = format!("{}/{}", path, TEMPLATE_NAME_INDEX);
    let invoice_path = format!("{}/{}", path, TEMPLATE_NAME_INVOICE);
    let letter_path = format!("{}/{}", path, RESULT_NAME_LETTER);
    let list_path = format!("{}/{}", path, RESULT_NAME_LIST);

    let letter_context = match Context::from_serialize(&letter) {
        Ok(context) => context,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen des Briefs (2).";
        }
    };

    let letter_result = match TEMPLATES.render(TEMPLATE_NAME_LETTER, &letter_context) {
        Ok(result) => result,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen des Briefs (3).";
        }
    };

    let html_letter_string = match serde_json::to_string_pretty(&letter_result) {
        Ok(letter) => letter,
        _ => return "Etwas ging schief beim Erstellen des Briefs (4).",
    };

    fs::write(letter_path, html_letter_string)
        .expect("Etwas ging schief beim Erstellen des Briefs (5).");

    let invoice_context = match Context::from_serialize(&invoice) {
        Ok(result) => result,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen der Rechnung (1).";
        }
    };

    let invoice_result = match TEMPLATES.render(TEMPLATE_NAME_INVOICE, &invoice_context) {
        Ok(result) => result,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen der Rechnung (2).";
        }
    };

    let html_invoice_string = match serde_json::to_string_pretty(&invoice_result) {
        Ok(invoice) => invoice,
        _ => return "Etwas ging schief beim Erstellen der Rechnung (3).",
    };

    fs::write(invoice_path, html_invoice_string)
        .expect("Etwas ging schief beim Erstellen der Rechnung (4).");

    let index_context = match Context::from_serialize(&index) {
        Ok(result) => result,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen der Übersicht (1).";
        }
    };

    let index_result = match TEMPLATES.render(TEMPLATE_NAME_INDEX, &index_context) {
        Ok(result) => result,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen der Übersicht (2).";
        }
    };

    let html_index_string = match serde_json::to_string_pretty(&index_result) {
        Ok(index) => index,
        _ => return "Etwas ging schief beim Erstellen der Übersicht (3).",
    };

    fs::write(index_path, html_index_string)
        .expect("Etwas ging schief beim Erstellen der Übersicht (4).");

    let list_context = match Context::from_serialize(&list) {
        Ok(result) => result,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen der Tipps";
        }
    };

    let list_result = match TEMPLATES.render(TEMPLATE_NAME_LIST, &list_context) {
        Ok(result) => result,
        Err(e) => {
            // log this: e.to_string().as_str();
            return "Etwas ging schief beim Erstellen der Tipps";
        }
    };

    // todo: create list
    // todo  also create a json file with the original data so we can easily adjust and recreate it
    // todo: 3. create html file for html/{file_name}/grundsteuereinspruch.html (tera)
    // todo: 4. create html file for html/{file_name}/rechnung.html             (tera)
    // todo: 5. create html file for html/{file_name}/index.html (contains links to letter + rechnung (pdf): /pdf/get/{file_name})
    // todo: 6. trigger create_pdf for {file_name}

    ""
}

async fn hello() -> &'static str {
    "v1"
}

async fn get_html(Path(params): Path<HashMap<String, String>>) -> &'static str {
    // todo return html pages based on params id and type
    let name = params.get("id");
    let page = params.get("type");

    ""
}

async fn get_pdf(Path(params): Path<HashMap<String, String>>) -> &'static str {
    // todo: get PDF by id
    // todo: 2. show pdf file by id, show error if not existing
    let name = params.get("id");
    let page = params.get("type");

    "v1"
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    /*let config = GovernorConfigBuilder::default()
    .per_second(4)
    .burst_size(2)
    .finish()
    .unwrap();*/

    let router = Router::new()
        .route("/", get(hello))
        // todo: rate limit this request (only 240 per minute per IP)
        .route("/pdf/:id", post(create_pdf))
        // todo rate limit this request (only 240 per minute per IP)
        .route("/html", post(create_html))
        // todo: rate limit this request (only 10 per minute per IP)
        .route("/pdf/:id/type/:type", get(get_pdf))
        .route("/page/:id/:type", get(get_html));
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
