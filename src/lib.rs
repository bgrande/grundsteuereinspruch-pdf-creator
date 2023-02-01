use std::collections::HashMap;
use std::{env, fmt, fs, thread, time};
use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::env::VarError;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result as AnyResult;
use axum::extract::{Path, Query, State};
use axum::{extract, routing::get, routing::post, Router, Extension};
use axum::http::StatusCode;
use axum::http::{Request};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, LocalResult, NaiveDateTime, Utc};
use chrono::LocalResult::Single;
use chrono::prelude::*;
use html2pdf::{run, CliOptions, Error as H2PError};
use sync_wrapper::SyncWrapper;
use shuttle_secrets::SecretStore;


use dotenv::dotenv;

pub(crate) mod send;
use crate::send::send::{get_email_config, send_email};

use rand::{Rng};
use ring::digest::{Context as RingContext, Digest, SHA256};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::{Error, Visitor};
use serde_json::{json, Value};
use shuttle_service::tracing::{debug, info};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{FromRow, Row};

use structopt::lazy_static::lazy_static;
use structopt::StructOpt;

use tera::{Context, Tera};
//use tower_request_id::{RequestIdLayer, RequestId};


//todo: use tower for rate limiting
const APP_TOKEN: &str = "846uoisdhgsdgszdog7846934634089hhuaip12xbo";
const FORM_ID: &str = "wvXAdv";
const FORM_NAME: &str = "Einspruch Grundsteuerbescheid";

const BASE_URL: &str = "https://app.grundsteuereinspruch.online";

const TEMPLATE_PATH: &str = "data/templates";
const TARGET_PATH: &str = "data/diedaten";
const MAPPING_PATH: &str = "data/diedaten/mapping";
const DB_PATH: &str = "data/db";

const SENDER_JSON: &str = "sender.json";

const TEMPLATE_NAME_INDEX: &str = "index.html";
const TEMPLATE_NAME_LETTER: &str = "letter.html";
const TEMPLATE_NAME_INVOICE: &str = "invoice.html";
const TEMPLATE_NAME_LIST: &str = "list.html";
const TEMPLATE_NAME_ERROR: &str = "error.html";
const TEMPLATE_NAME_MAPPING: &str = "loading.html";

const RESULT_NAME_LETTER: &str = "Grundsteuereinspruch.pdf";
const RESULT_NAME_INVOICE: &str = "Grundsteuereinspruch-Rechnung.pdf";
const RESULT_NAME_LIST: &str = "Grundsteuereinspruch-Liste.pdf";



lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let template_path = format!(
            "{}/*.html",
            TEMPLATE_PATH
        );

        let mut tera = match Tera::new(template_path.as_str()) {
            Ok(t) => t,
            Err(e) => {
                info!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".htm"]);
        //tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

fn parse_value<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AnyType<'a> {
        Str(&'a str),
        U64(u64),
        Vec(Vec<String>),
        Bool(bool),
        None,
    }

    Ok(match AnyType::deserialize(deserializer)? {
        AnyType::Str(v) => Some(vec![v.to_string()]),
        AnyType::U64(v) => Some(vec![v.to_string()]),
        AnyType::Vec(v) => Some(v),
        AnyType::Bool(v) => Some(vec![v.to_string()]),
        AnyType::None => None,
    })
}
#[derive(FromRow, Debug)]
struct TaxOffice {
    name: String,
    zip: String,
    city: String,
    street: String,
    number: String,
}

#[derive(Serialize, Deserialize)]
struct Sender {
    first_name: String,
    last_name: String,
    company_name: String,
    street: String,
    number: String,
    zip: String,
    city: String,
    email: String,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FormFieldOption {
    id: String,
    text: String,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FormField {
    key: String,
    label: String,
    #[serde(rename = "type")]
    form_type: String,
    #[serde(deserialize_with="parse_value")]
    value: Option<Vec<String>>,
    options: Option<Vec<FormFieldOption>>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuestionData {
    response_id: String,
    submission_id: String,
    respondent_id: String,
    form_id: String,
    form_name: String,
    created_at: String,
    fields: Vec<FormField>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuestionResult {
    event_id: String,
    event_type: String,
    created_at: String,
    data: QuestionData,
}

#[derive(Debug)]
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
    sent_date: String,
    subject_text: String,
    additional_senders: bool,
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
    sender_first_name: String,
    sender_last_name: String,
    sender_company_name: String,
    sender_street: String,
    sender_number: String,
    sender_zip: String,
    sender_city: String,
    sender_email: String,
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
    sent_date: String,
    deadline_date: String,
}
#[derive(Serialize)]
struct List {
    first_name: String,
    last_name: String,
    date: String,
    file_id: String,
    sent_date: String,
    deadline_date: String,
}
#[derive(Serialize)]
struct EMail {
    first_name: String,
    last_name: String,
    email: String,
    date: String,
    link: String,
    sent_date: String,
    deadline_date: String,
}

#[derive(Serialize)]
struct Mapping {
    first_name: String,
    last_name: String,
    email: String,
    mapping_id: String,

}

fn get_mapping_hash(respondent: &str, submission: &str, email: &str) -> u64 {
    let to_hash = ToHash {
        value: format!("{}{}{}", submission, respondent, email),
    };

    calculate_hash(to_hash)
}

fn create_random_id() -> AnyResult<String> {
    let mut random_gen = rand::thread_rng();
    let mut context = RingContext::new(&SHA256);

    // add a lot of random numbers
    for _i in 1..250 {
        let data = random_gen.gen();
        context.update(&[data]);
    }

    let digest = context.finish();
    let file_id = data_encoding::HEXLOWER.encode(digest.as_ref());

    Ok(file_id)
}

fn get_target_path(id: String, target_path: String) -> AnyResult<String> {
    let current_dir_binding = env::current_dir()?;
    let current_dir_env = match current_dir_binding.to_str() {
        Some(dir) => dir,
        None => "",
    };

    let current_dir = String::from(current_dir_env);
    let html_path = format!("{}/{}/{}", current_dir, target_path, id);

    Ok(html_path)
}

fn create_path(file_id: String, target_path: String) -> AnyResult<String> {
    let path = get_target_path(file_id, target_path)?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn get_date_from_micro(date: &str) -> Option<NaiveDateTime> {
    info!("sent date: {}", date.to_string());
    
    let split = date.split(".");
    let split_vec = split.collect::<Vec<&str>>();
    let sent = NaiveDateTime::parse_from_str(
        format!("{}+00:00", split_vec[0]).as_str(),
        "%Y-%m-%dT%H:%M:%S%z",
    );

    let sent_date_time = match sent {
        Ok(date_time) => date_time,
        Err(e) => {
            info!("date conversion issue: {}", e.to_string());
            return None;
        },
    };

    Some(sent_date_time)
}

fn html2pdf(path: String, result_path: String) -> std::result::Result<(), H2PError> {
    let options = Vec::from(["input", path.as_str(), "--output", result_path.as_str()]);
    let opt = CliOptions::from_iter(options);

    // Let's go
    run(opt)
}

fn to_result(from_2pdf: ()) -> bool {
    return match from_2pdf {
        () => true,
    };
}

fn is_valid_payload(payload: &QuestionResult) -> bool {
    let date = get_date_from_micro(payload.created_at.as_str());

    let sent_date_time = match date {
        Some(date_time) => date_time.timestamp(),
        None => return false,
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

fn get_sender_object() -> AnyResult<Sender> {
    let file_path = format!("{}/{}", DB_PATH, SENDER_JSON);

    let data = fs::read_to_string(file_path);
    let data_string = data?;
    let sender: Sender = serde_json::from_str(data_string.as_str())?;

    Ok(sender)
}

#[derive(Hash)]
struct ToHash {
    value: String,
}

fn calculate_hash(to_hash: ToHash) -> u64 {
    let mut hasher = DefaultHasher::new();
    to_hash.hash(&mut hasher);
    hasher.finish()
}

fn generate_customer_id(first_name: &String, last_name: &String, email: &String) -> String {
    let hash_object = ToHash {
        value: format!("{}{}{}{}", Utc::now().format("%Y%m%dH%M%S"), first_name, last_name, email),
    };

    let hashed = calculate_hash(hash_object).to_string();

    return format!(
        "{}",
        hashed.split_at(7).0
    )
}

fn generate_invoice_id(customer_id: &String) -> String {
    let hash_object = ToHash {
        value: format!("{}{}", Utc::now().format("%Y%m%dH%M%S%f"), customer_id),
    };

    let hashed = calculate_hash(hash_object).to_string();

    return format!(
        "{}-{}",
        Utc::now().format("%Y"),
        hashed.split_at(6).0
    )
}

async fn get_tax_office_query(zip: &String, name: &String) -> AnyResult<TaxOffice, anyhow::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://data/db/tax_offices.db").await?;

    info!("using zip {} and name {}", zip, name);
    
    let row = sqlx::query_as::<_, TaxOffice>(
            "
SELECT
    ad.city,
    ad.street,
    ad.number,
    ad.zip,
    tao.name
FROM address ad
JOIN address_to_tax_office atto ON ad.id = atto.address_id
JOIN tax_office tao ON atto.tax_office_id = tao.id
WHERE
    zip = ?
    AND name = ?
"
        )
        .bind(zip)
        .bind(name)
        .fetch_one(&pool).await?;

    Ok(row)
}

fn get_value_from_option(options: &Option<Vec<FormFieldOption>>, vec_val: Vec<String>) -> Vec<String> {
    let binding = Vec::new();
    let options = match options.to_owned() {
        Some(option) => option,
        None => &binding,
    };

    let mut list: Vec<String> = vec![];

    for value in vec_val {
        match options.iter().find(|&item| item.id == value) {
            Some(option) => {
                // only add if we don't have the custom (Sonstiges) option
                if option.id != "10b80582-5b19-4906-bdb7-c656ffc22ba9" {
                    list.push(option.text.clone())
                }
            },
            None => (),
        };
    }

    return list;
}

fn get_naive_date_from_string(date_value: String) -> AnyResult<NaiveDateTime, anyhow::Error> {
    Ok(NaiveDateTime::parse_from_str(
        format!("{}{}", date_value, "T00:05:00+00:00").as_str(),
        "%Y-%m-%dT%H:%M:%S%z"
    )?)
}

fn get_formatted_date_from_string(date: String, format: &str) -> AnyResult<String> {
    let naive_date_result = get_naive_date_from_string(date)?;
    let utc_naive_date = Utc.from_local_datetime(&naive_date_result);

    return Ok(match utc_naive_date {
        Single(date_time) => date_time.format_localized(format, Locale::de_DE).to_string(),
        LocalResult::Ambiguous(_, _) => {
            info!("date conversion issue: ambiguous");
            "".to_string()
        },
        LocalResult::None => {
            info!("date conversion issue: not existing");
            "".to_string()
        },
    });
}

/*
fn hash_email(email: &str) -> AnyResult<String> {
}
 */

fn get_html_page(name: &str, page: &str) -> AnyResult<String> {
    let file_path = format!("{}/{}/{}.html", TARGET_PATH, name, page);
    let data = fs::read_to_string(file_path);
    let data_string = data?;

    Ok(data_string.clone())
}

fn create_pdf_by_id(base_path: String) -> Option<bool> {
    // todo 1. create another random hash? Or use the given one
    // todo 2. get html files by html/{file_name}/{grundsteuereinspruch|invoice}.html -> convert to pdf/{file_name}/{id}.pdf + pdf/{file_name}/{id}_invoice.pdf
    // todo 3. send invoice link to me via email
    // todo 4. send overview link to me via email
    let html2pdf_letter = match html2pdf(
        format!("{}/{}", base_path, TEMPLATE_NAME_LETTER),
        format!("{}/{}", base_path, RESULT_NAME_LETTER)
    ) {
        Ok(result) => result,
        Err(e) => {
            info!("sth. went wrong creating the pdf for the letter: {}", e.to_string());
            return None
        },
    };

    let html2pdf_invoice = match html2pdf(
        format!("{}/{}", base_path, TEMPLATE_NAME_INVOICE),
        format!("{}/{}", base_path, RESULT_NAME_INVOICE)
    ) {
        Ok(result) => result,
        Err(e) => {
            info!("sth. went wrong creating the pdf for the invoice: {}", e.to_string());
            return None
        },
    };

    let html2pdf_list = match html2pdf(
        format!("{}/{}", base_path, TEMPLATE_NAME_LIST),
        format!("{}/{}", base_path, RESULT_NAME_LIST)
    ) {
        Ok(result) => result,
        Err(e) => {
            info!("sth. went wrong creating the pdf for the tipps: {}", e.to_string());
            return None
        },
    };

    Some(to_result(html2pdf_letter) && to_result(html2pdf_invoice) && to_result(html2pdf_list))
}

// todo: this is a separate pdf creation endpoint which should use shared logic from the create_html endpoint
async fn create_pdf(Path(params): Path<HashMap<String, String>>) -> &'static str {
    let creator_id = match params.get("id") {
        Some(id) => id,
        _ => "",
    };
    let create_result = create_pdf_by_id(creator_id.clone().to_string());

    return match create_result {
        // todo: redirect to index
        Some(result) => "success",
        None => "Etwas ging schief beim Erstellen des PDFs",
    }
}

async fn create_html(
    State(state): State<Arc<AppState>>,
    Path(params): Path<HashMap<String, String>>,
    extract::Json(payload): extract::Json<QuestionResult>
) -> &'static str  {
    let file_id = match create_random_id() {
        Ok(id) => id,
        Err(e) =>  {
            info!("Brieferstellung, id creation: {}", e.to_string());
            return "Etwas lief schief beim Erstellen des Briefs (1).";
            // todo add kontakt-link (use error template)
        }
    };

    let base_path = match create_path(file_id.clone(), TARGET_PATH.to_string()) {
        Ok(path) => path,
        Err(e) => {
            info!("Brieferstellung, path creation: {}", e.to_string());
            return "Etwas lief schief beim Erstellen des Briefs (1).";
            // todo add kontakt-link (use error template)
        }
    };

    let payload_json_path = format!("{}/{}", base_path, format!("{}.json", file_id));

    let payload_string = match serde_json::to_string_pretty(&payload) {
        Ok(index) => index,
        Err(e) => {
            info!("error creating json for the payload for file_id {}: {}", file_id, e.to_string());
            "".to_string()
        },
    };

    match fs::write(payload_json_path, payload_string) {
        Ok(_) => {},
        Err(e) => info!("error writing json for the payload for file_id {}: {}", file_id, e.to_string()),
    }

    let is_payload_valid = is_valid_payload(&payload);

    if !is_payload_valid {
        // todo activate and test this check
        //return "The payload sent is invalid";
    }

    let sender = match get_sender_object() {
        Ok(object) => object,
        Err(e) => {
            info!("error getting the sender data: {}", e.to_string());
            return "Etwas ging schief bei der Brieferstellung (ß)";
        }
    };

    let date_now = Utc::now().format_localized("%e. %B %Y", Locale::de_DE).to_string();

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
        sender_names_intro: "Eigentümer".to_string(),
        sender_names: vec![],
        receiver_office_name: "".to_string(),
        receiver_office_address: "".to_string(),
        receiver_office_zip: "".to_string(),
        receiver_office_city: "".to_string(),
        objection_subjects: vec![],
        objection_subject_start_dates: vec![],
        objection_subject_reasons: vec![],
        date_created: date_now.clone(),
        sent_date: "".to_string(),
        subject_text: "Einspruch gegen den Bescheid zur Feststellung des ".to_string(),
        additional_senders: false,
    };
    
    let mut invoice = Invoice {
        sender_first_name: sender.first_name,
        sender_last_name: sender.last_name,
        sender_company_name: sender.company_name,
        sender_street: sender.street,
        sender_number: sender.number,
        sender_zip: sender.zip,
        sender_city: sender.city,
        sender_email: sender.email,
        first_name: "".to_string(),
        last_name: "".to_string(),
        street: "".to_string(),
        number: "".to_string(),
        zip: "".to_string(),
        city: "".to_string(),
        email: "".to_string(),
        date: date_now.clone(),
        invoice_id: "".to_string(),   // todo: generate (based on random + customer_id + date)
        customer_id: "".to_string(),  // todo: generate (based on first_name, last_name, date)
        subject_text: "Ihre Rechnung".to_string(),
        payment: Payment {
            price: "".to_string(),
            currency: "".to_string(),
            name: "".to_string(),
            email: "".to_string(),
            link: "".to_string(),
        },
        invoice_objects: vec![
            "Erstellung eines individuellen Einspruchsbriefes an Ihr Finanzamt       |       1,26 €".to_string(),
            "                                                        MwSt. 19%       |       0,24 €".to_string(),
            "                                                           Gesamt       |       1,50 €".to_string(),
        ],
    };
    let mut index = Index {
        first_name: "".to_string(),
        last_name: "".to_string(),
        date: "".to_string(),
        file_id: file_id.clone(),
        sent_date: "".to_string(),
        deadline_date: "".to_string(),
    };
    let mut list = List {
        first_name: "".to_string(),
        last_name: "".to_string(),
        date: "".to_string(),
        file_id: "".to_string(),
        sent_date: "".to_string(),
        deadline_date: "".to_string(),
    };
    let mut email = EMail {
        first_name: "".to_string(),
        last_name: "".to_string(),
        email: "".to_string(),
        date: "".to_string(),
        link: "".to_string(),
        sent_date: "".to_string(),
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
    let mut meta_no_warranty: FormFieldMeta = FormFieldMeta {
        key: "".to_string(),
        value: "".to_string(),
    };

    // todo write the whole payload into JSON?

    for field in payload.data.fields {
        let value = field.value.to_owned();

        let current_value = match value {
            Some(value) => value,
            None => continue,
        };

        // meta_ref, meta_origin are for analytics, origin and token for another API check as well
        if field.key == "question_3xzMoo_81f3e592-de5c-48f2-b459-b494d65dfc65" {
            meta_reference = FormFieldMeta {
                key: field.key.clone(),
                value: current_value[0].clone(),
            };
        }

        if field.key == "question_3xzMoo_60b2c1c7-03d6-43ae-9266-fd5c29143450" {
            meta_origin_page = FormFieldMeta {
                key: field.key.clone(),
                value: current_value[0].clone(),
            };
        }

        if field.key == "question_3xzMoo_80705223-756a-4a8d-aa1e-e8b0147a977c" {
            meta_token = FormFieldMeta {
                key: field.key.clone(),
                value: current_value[0].clone(),
            };
        }

        if field.key == "question_wzdyR1" {
            let first_name_val = current_value[0].clone();
            
            letter.first_name = first_name_val.clone();
            index.first_name = first_name_val.clone();
            list.first_name = first_name_val.clone();
            invoice.first_name = first_name_val.clone();
            email.first_name = first_name_val.clone();
        }

        if field.key == "question_m6p5BP" {
            let last_name_val = current_value[0].clone();

            letter.last_name = last_name_val.clone();
            index.last_name = last_name_val.clone();
            list.last_name = last_name_val.clone();
            invoice.last_name = last_name_val.clone();
            email.last_name = last_name_val.clone();
        }

        if field.key == "question_w7p0e6" {
            let street_val = current_value[0].clone();

            letter.street = street_val.clone();
            invoice.street = street_val.clone();
        }

        if field.key == "question_mV47y6" {
            let number_val = current_value[0].clone();

            letter.number = number_val.clone();
            invoice.number = number_val.clone();
        }

        if field.key == "question_nPGQyx" {
            let zip_val = current_value[0].clone();

            letter.zip = zip_val.clone();
            letter.receiver_office_zip = zip_val.clone();
            invoice.zip = zip_val.clone();
        }

        if field.key == "question_3ENVY2" {
            let city_val = current_value[0].clone();

            letter.city = city_val.clone();
            invoice.city = city_val.clone();
        }

        if field.key == "question_wazO6q" {
            let email_val = current_value[0].clone();

            letter.email = email_val.clone();
            invoice.email = email_val.clone();
            email.email = email_val.clone();
        }

        if field.key == "question_w2RJMg" {
            letter.phone = current_value[0].clone();
        }

        if field.key == "question_nGQE8p" {
            letter.sender_names.push(current_value[0].clone());
        }

        if field.key == "question_wAjR0o" {
            let max_count_val = current_value[0].clone();

            letter.max_sender_count = match max_count_val.parse() {
                Ok(number) => number,
                Err(_) => 0,
            };
        }

        let sender_name = current_value[0].clone();
        if field.key == "question_nGQE8p" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_mV472J" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_waaJ7v" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_m6PD7B" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_nPrDxB" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_nrYB42" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_3jzbRJ" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_3xzjqv" {
            letter.sender_names.push(sender_name.clone());
        }
        if field.key == "question_3NPDjO" {
            letter.sender_names.push(sender_name.clone());
        }
        
        if letter.sender_names.len() > 1 {
            letter.additional_senders = true;
        }

        if field.key == "question_mJWZ0r" {
            let sent_date_chrono = get_naive_date_from_string(current_value[0].clone());

            let tax_office_sent_date_val = match sent_date_chrono {
                Ok(date_time) => date_time,
                Err(e) => {
                    info!("date conversion issue: {}", e);
                    return "Das Datum des Bescheidbriefes ist falsch"
                },
            };

            let utc_sent_date = Utc.from_local_datetime(&tax_office_sent_date_val);

            let formatted_sent_date = match utc_sent_date {
                Single(date_time) => date_time.format_localized("%d.%m.%Y", Locale::de_DE).to_string(),
                LocalResult::Ambiguous(_, _) => {
                    info!("date conversion issue: ambiguous");
                    "".to_string()
                },
                LocalResult::None => {
                    info!("date conversion issue: not existing");
                    "".to_string()
                },
            };

            let deadline_date = match sent_date_chrono {
                Ok(date_time) => date_time + Duration::weeks(4),
                Err(e) => {
                    info!("date conversion issue: {}", e);
                    return "Das Datum des Bescheidbriefes ist falsch"
                },
            };

            let utc_deadline = Utc.from_local_datetime(&deadline_date);
            let formatted_deadline = match utc_deadline {
                Single(date_time) => date_time.format_localized("%e. %B %Y", Locale::de_DE).to_string(),
                LocalResult::Ambiguous(_, _) => {
                    info!("date conversion issue: ambiguous");
                    "".to_string()
                },
                LocalResult::None => {
                    info!("date conversion issue: not existing");
                    "".to_string()
                },
            };

            letter.sent_date = formatted_sent_date.clone();

            index.sent_date = formatted_sent_date.clone();
            index.deadline_date = formatted_deadline.clone();

            list.sent_date = formatted_sent_date.clone();
            list.deadline_date = formatted_deadline.clone();

            email.sent_date = formatted_sent_date.clone();
            email.deadline_date = formatted_deadline.clone();
        }

        // Aktenzeichen
        if field.key == "question_wgkx4P" {
            letter.reference_number = current_value[0].clone();
        }

        if field.key == "question_nrk9WX" {
            letter.receiver_office_zip = current_value[0].clone();
        }

        if field.key == "question_wMy51M" && field.options.is_some() {
            letter.receiver_office_name = get_value_from_option(&field.options, current_value.clone())[0].clone();
        }

        let check_val = current_value[0].clone();

        // this is Einspruch für Grundsteuerwertbescheid
        if field.key == "question_nWjbDJ_11f917de-4e6a-4290-838e-9d194afd11af"
            && check_val.to_owned().parse() == Ok(true)
        {
            letter
                .objection_subjects
                .push("Grundsteuerwertbescheid".to_string());
        }
        // this is Einspruch für Grundsteuermessbescheid
        if field.key == "question_nWjbDJ_73a81f7f-2dbe-4fcd-9573-38f98596049f"
            && check_val.to_owned().parse() == Ok(true)
        {
            letter
                .objection_subjects
                .push("Grundsteuermessbescheid".to_string());
        }
        // this is Einspruch für Grundsteuerwertbescheid
        if field.key == "question_w8Rgel" && !check_val.to_owned().is_empty() {
            let formatted_naive_date = match get_formatted_date_from_string(check_val.clone(), "%d.%m.%Y") {
                Ok(value) => value,
                _ => "01.01.2025".to_string()
            };

            letter
                .objection_subject_start_dates
                .push(formatted_naive_date);
        }
        // this is Einspruch für Grundsteuermessbescheid
        if field.key == "question_n0DgP9" && !check_val.to_owned().is_empty() {
            let formatted_naive_date = match get_formatted_date_from_string(check_val.clone(), "%d.%m.%Y") {
                Ok(value) => value,
                _ => "01.01.2025".to_string()
            };

            letter
                .objection_subject_start_dates
                .push(formatted_naive_date);
        }

        if field.key == "question_mRjGNv" && !check_val.to_owned().is_empty() && field.options.is_some() {
            letter.objection_subject_reasons = get_value_from_option(&field.options, current_value.clone());
        }

        if field.key == "question_npkqkZ" && !check_val.to_owned().is_empty() {
            letter
                .objection_subject_reasons
                .push(check_val.clone());
        }

        if field.key == "question_3ENVAL_price" {
            invoice.payment.price = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_currency" {
            invoice.payment.currency = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_name" {
            invoice.payment.name = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_email" {
            invoice.payment.email = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_link" {
            invoice.payment.link = current_value[0].clone();
        }

        if field.key == "question_nrYOOl_7200bf88-f16a-4384-9d85-06c7b97b6a4a" {
            meta_start_now = FormFieldMeta {
                key: field.key.clone(),
                value: current_value[0].clone(),
            };
        }
        if field.key == "question_w4O77k_ba1c873b-1bbc-4b00-94a4-0cec5d3b3655" {
            meta_no_revocation = FormFieldMeta {
                key: field.key.clone(),
                value: current_value[0].clone(),
            };
        }

        if field.key == "question_m697JB_88b38203-b196-4e22-b4da-507b5e16eb6d" {
            meta_no_warranty = FormFieldMeta {
                key: field.key.clone(),
                value: current_value[0].clone(),
            };
        }
    }

    let tax_office_address = get_tax_office_query(&letter.receiver_office_zip, &letter.receiver_office_name);
    let tax_office_address_object = match tax_office_address.await {
        Ok(address) => address,
        Err(e) => {
            info!("sth. went wrong getting the tax office address: {}", e);
            return "Die angegebenen Finanzamtdaten waren vermutlich fehlerhaft. Das Finanzamt konnte nicht in unserer Datenbank gefunden werden.";
        }
    };

    info!("tax office address: {:?}", tax_office_address_object);
    letter.receiver_office_name = tax_office_address_object.name;
    letter.receiver_office_zip = tax_office_address_object.zip;
    letter.receiver_office_city = tax_office_address_object.city;
    letter.receiver_office_address = format!("{} {}", tax_office_address_object.street, tax_office_address_object.number);

    info!("start_now: {:?}", &meta_start_now);
    info!("no_warranty: {:?}", &meta_no_warranty);
    info!("origin_page: {:?}", &meta_origin_page);
    info!("token: {:?}", &meta_token);

    if meta_start_now.value.parse() == Ok(false) || meta_no_revocation.value.parse() == Ok(false) {
        return "Die Zustimmung zur Ausführung des Vertrags vor Ablauf der Widerrufsfrist und/oder den Verlust des Widerrufsrechts dadurch fehlt.";
    }
    if meta_no_warranty.value.parse() == Ok(false) {
        return "Es fehlt die Zustimmung zum Garantieausschluss";
    }
    if meta_origin_page.value != "/fragebogen.html" || meta_token.value != APP_TOKEN {
        return "Der Aufruf war fehlerhaft!";
    }

    let mapping_hash = get_mapping_hash(payload.data.respondent_id.as_str(), payload.data.submission_id.as_str(), letter.email.as_str());

    let mapping_base_path = match create_path(mapping_hash.to_string(), MAPPING_PATH.to_string()) {
        Ok(path) => path,
        Err(e) => {
            info!("Brieferstellung, path creation: {}", e.to_string());
            return "Etwas lief schief beim Erstellen des Briefs (1).";
            // todo add kontakt-link (use error template)
        }
    };

    let base_url = match env::var("BASE_URL") {
        Ok(val) => val,
        Err(_) => BASE_URL.to_string(),
    };
    let link = format!("{}/page/{}/index", base_url, file_id);

    let mapping_path = format!("{}/index.html", mapping_base_path);

    let mut mapping_context = Context::new();
    mapping_context.insert("file_id", &file_id);
    mapping_context.insert("base_url", &base_url);

    let mapping_result = match TEMPLATES.render(TEMPLATE_NAME_MAPPING, &mapping_context) {
        Ok(result) => result,
        Err(e) => {
            info!("Mapping redirect, template rendering: {}", e.to_string());
            return "Etwas ging schief beim Erstellen des Briefs (0).";
        }
    };
    match fs::write(mapping_path, mapping_result) {
        Ok(_) => {},
        Err(_) => info!("Mapping redirect creation failed.")
    }

    invoice.customer_id = generate_customer_id(&letter.first_name, &letter.last_name, &letter.email);
    invoice.invoice_id = generate_invoice_id(&invoice.customer_id);

    let index_path = format!("{}/{}", base_path, TEMPLATE_NAME_INDEX);
    let invoice_path = format!("{}/{}", base_path, TEMPLATE_NAME_INVOICE);
    let letter_path = format!("{}/{}", base_path, TEMPLATE_NAME_LETTER);
    let list_path = format!("{}/{}", base_path, TEMPLATE_NAME_LIST);

    let letter_context = match Context::from_serialize(&letter) {
        Ok(context) => context,
        Err(e) => {
            info!("Brieferstellung, context serializing: {}", e.to_string());
            return "Etwas ging schief beim Erstellen des Briefs (2).";
        }
    };

    let letter_result = match TEMPLATES.render(TEMPLATE_NAME_LETTER, &letter_context) {
        Ok(result) => result,
        Err(e) => {
            info!("Brieferstellung, template rendering: {}", e.to_string());
            return "Etwas ging schief beim Erstellen des Briefs (3).";
        }
    };

    match fs::write(letter_path, letter_result) {
        Ok(_) => {},
        Err(_) => info!("Etwas ging schief beim Erstellen des Briefs (3).")
    }

    let invoice_context = match Context::from_serialize(&invoice) {
        Ok(result) => result,
        Err(e) => {
            info!("Rechnung, context serializing: {}", e.to_string());
            return "Etwas ging schief beim Erstellen der Rechnung (1).";
        }
    };

    let invoice_result = match TEMPLATES.render(TEMPLATE_NAME_INVOICE, &invoice_context) {
        Ok(result) => result,
        Err(e) => {
            info!("Rechnung, template rendering: {}", e.to_string());
            return "Etwas ging schief beim Erstellen der Rechnung (2).";
        }
    };

    match fs::write(invoice_path, invoice_result) {
        Ok(_) => {},
        Err(_) => info!("Etwas ging schief beim Erstellen der Rechnung (3).")
    }

    let index_context = match Context::from_serialize(&index) {
        Ok(result) => result,
        Err(e) => {
            info!("Index, context serializing: {}", e.to_string());
            return "Etwas ging schief beim Erstellen der Übersicht (1).";
        }
    };

    let index_result = match TEMPLATES.render(TEMPLATE_NAME_INDEX, &index_context) {
        Ok(result) => result,
        Err(e) => {
            info!("Index, template rendering: {}", e.to_string());
            return "Etwas ging schief beim Erstellen der Übersicht (2).";
        }
    };

    match fs::write(index_path, index_result) {
        Ok(_) => {},
        Err(_) => info!("Etwas ging schief beim Erstellen der Übersicht (3).")
    }

    let list_context = match Context::from_serialize(&list) {
        Ok(result) => result,
        Err(e) => {
            info!("List, context serializing: {}", e.to_string());
            return "Etwas ging schief beim Erstellen der Tipps (1)";
        }
    };

    let list_result = match TEMPLATES.render(TEMPLATE_NAME_LIST, &list_context) {
        Ok(result) => result,
        Err(e) => {
            info!("List, template rendering: {}", e.to_string());
            return "Etwas ging schief beim Erstellen der Tipps (2)";
        }
    };

    match fs::write(list_path, list_result) {
        Ok(_) => {},
        Err(_) => info!("Etwas ging schief beim Erstellen der Tipps (3).")
    }

    let pdf_creation_result = match create_pdf_by_id(base_path) {
        // todo: redirect to index
        Some(result) => "success",
        None => "Etwas ging schief beim Erstellen des PDFs",
    };

    match send_email(&letter, link, get_email_config(state.email_user.clone(), state.email_pass.clone())) {
        // ok is just fine
        Ok(res) => {},
        Err(e) => info!("unexpected error while sending email: {}", e)
    };

    return pdf_creation_result;
}

async fn hello() -> &'static str {
    "v1"
}

async fn get_html(Path(params): Path<HashMap<String, String>>) -> axum::response::Html<String> {
    let name = match params.get("id") {
        None => "",
        Some(val) => val.as_str(),
    };
    let page = match params.get("type") {
        None => "",
        Some(val) => val.as_str(),
    };

    let allowed_types = vec!["index"];

    if !allowed_types.contains(&page) {
        // todo use error page
        info!("trying to get page of type {} which doesn't exist.", &page);
        return "This page does not exist!".to_string().into();
    }

    let page_result = get_html_page(name, page);
    
    return match page_result {
        Ok(result) => result.into(),
        Err(e) => {
            info!("couldn't get page with name {}: {}", &name, e.to_string());
            "The page does not exist!".to_string().into()
        }
    }
}

async fn get_result_page(Query(params): Query<HashMap<String, String>>) -> axum::response::Html<String> {
    let email = match params.get("email") {
        None => "",
        Some(val) => val.as_str(),
    };
    let submission_id = match params.get("subid") {
        None => "",
        Some(val) => val.as_str(),
    };
    let respondent_id = match params.get("resp") {
        None => "",
        Some(val) => val.as_str(),
    };

    if !email.contains("@") {
        // todo: return with error
    }
    
    let name = "mapping";
    let page = get_mapping_hash(respondent_id, submission_id, email).clone().to_string();

    let sleep_time = time::Duration::from_millis(4000);
    thread::sleep(sleep_time);

    let page_result = get_html_page(name, &format!("{}/index", page.clone()));

    return match page_result {
        Ok(result) => result.into(),
        Err(e) => {
            info!("couldn't get page with name {}: {}", &name, e.to_string());
            "The page does not exist!".to_string().into()
        }
    }
}

async fn get_pdf(Path(params): Path<HashMap<String, String>>) -> &'static str {
    // todo: get PDF by id
    // todo: 2. show pdf file by id, show error if not existing
    let name = params.get("id");
    let page = params.get("type");
    // allowed types: letter, invoice, list

    "v1"
}

struct AppState {
    email_user: String,
    email_pass: String,
}

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
