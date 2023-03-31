use std::{env, fs, thread, time};
use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{extract};
use axum::body::StreamBody;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse};
use chrono::{Duration, LocalResult, Utc};
use chrono::LocalResult::Single;
use chrono::prelude::*;
use log::{debug, error, info, warn};
use tokio_util::io::ReaderStream;

use crate::crypt::*;
use crate::db::*;
use crate::pdf::*;
use crate::helper::*;
use crate::html::*;
use crate::form::*;
use crate::objects::*;
use crate::config::*;
use crate::form::QuestionResult;

use crate::send::send::{get_email_config, send_email, send_email_owner};

use structopt::lazy_static::lazy_static;
use tera::{Context, Tera};
use tokio::fs::File;

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


pub struct AppState {
    pub email_user: String,
    pub email_pass: String,
}

// todo: this is a separate pdf creation endpoint which should use shared logic from the create_html endpoint
pub async fn create_pdf(Path(params): Path<HashMap<String, String>>) -> &'static str {
    let creator_id = match params.get("id") {
        Some(id) => id,
        _ => "",
    };
    let create_result = create_pdf_by_id(creator_id.clone().to_string());

    return match create_result {
        // todo: redirect to index
        Some(_) => "success",
        None => "Etwas ging schief beim Erstellen des PDFs",
    }
}

pub async fn create_html(
    State(state): State<Arc<AppState>>,
    extract::Json(payload): extract::Json<QuestionResult>,
) -> impl IntoResponse  {
    let pay_after = false; // todo for other payments

    let headers = [(header::CONTENT_TYPE, "text/html")];

    let file_id = match create_random_id() {
        Ok(id) => id,
        Err(e) =>  {
            error!("Brieferstellung, id creation: {}", e);
            let string = get_error_page("Etwas lief schief beim Erstellen des Briefs (id).");
            return Err((StatusCode::BAD_REQUEST, headers, string))
        }
    };

    let base_path = match create_path(file_id.clone(), TARGET_PATH.to_string()) {
        Ok(path) => path,
        Err(e) => {
            error!("Brieferstellung, path creation: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (base_path).")));
        }
    };

    let payload_json_path = format!("{}/{}", base_path, format!("{}.json", file_id));

    let payload_string = match serde_json::to_string_pretty(&payload) {
        Ok(index) => index,
        Err(e) => {
            error!("error creating json for the payload for file_id {}: {}", file_id, e);
            "".to_string()
        },
    };

    match fs::write(payload_json_path, payload_string) {
        Ok(_) => {},
        Err(e) => error!("error writing json for the payload for file_id {}: {}", file_id, e),
    }

    let is_payload_valid = is_valid_payload(&payload);

    if !is_payload_valid {
        // todo test this!
        error!("Payload was invalid!");
        return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (payload).")));
    }

    let sender = match get_sender_object(DB_PATH, SENDER_JSON) {
        Ok(object) => object,
        Err(e) => {
            error!("error getting the sender data: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (sender_data).")));
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
        multiple_objection_subjects: false,
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

        if field.key == "question_wAjR0o" {
            let max_count_val = current_value[0].clone();

            letter.max_sender_count = match max_count_val.parse() {
                Ok(number) => number,
                Err(_) => 0,
            };
        }

        let sender_name = current_value[0].clone();
        if field.key == "question_nGQE8p" {
            letter.sender_names.push(format!("{} {}", letter.first_name, letter.last_name));
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

        if letter.objection_subjects.len() > 1 {
            letter.multiple_objection_subjects = true;
        }

        if field.key == "question_mJWZ0r" {
            let sent_date_chrono = get_naive_date_from_string(current_value[0].clone());

            let tax_office_sent_date_val = match sent_date_chrono {
                Ok(date_time) => date_time,
                Err(e) => {
                    error!("date conversion issue: {}", e);
                    return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Das Datum des Bescheidbriefes ist falsch.")));
                },
            };

            let utc_sent_date = Utc.from_local_datetime(&tax_office_sent_date_val);

            let formatted_sent_date = match utc_sent_date {
                Single(date_time) => date_time.format_localized("%d.%m.%Y", Locale::de_DE).to_string(),
                LocalResult::Ambiguous(_, _) => {
                    error!("date conversion issue: ambiguous");
                    "".to_string()
                },
                LocalResult::None => {
                    error!("date conversion issue: not existing");
                    "".to_string()
                },
            };

            let deadline_date = match sent_date_chrono {
                Ok(date_time) => date_time + Duration::weeks(4),
                Err(e) => {
                    error!("date conversion issue: {}", e);
                    return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Das Datum des Bescheidbriefes ist falsch. (deadline)")));
                },
            };

            let utc_deadline = Utc.from_local_datetime(&deadline_date);
            let formatted_deadline = match utc_deadline {
                Single(date_time) => date_time.format_localized("%e. %B %Y", Locale::de_DE).to_string(),
                LocalResult::Ambiguous(_, _) => {
                    error!("date conversion issue: ambiguous");
                    "".to_string()
                },
                LocalResult::None => {
                    error!("date conversion issue: not existing");
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
                .push("Grundsteuerwert".to_string());
        }
        // this is Einspruch für Grundsteuermessbescheid
        if field.key == "question_nWjbDJ_73a81f7f-2dbe-4fcd-9573-38f98596049f"
            && check_val.to_owned().parse() == Ok(true)
        {
            letter
                .objection_subjects
                .push("Grundsteuermessbetrag".to_string());
        }
        // this is Einspruch für Grundsteuerwertbescheid
        if field.key == "question_w8Rgel" && letter.objection_subjects.contains(&"Grundsteuerwert".to_string()) {
            let formatted_naive_date = match get_formatted_date_from_string(check_val.clone(), "%d.%m.%Y") {
                Ok(value) => value,
                _ => "01.01.2025".to_string()
            };

            letter
                .objection_subject_start_dates
                .push(formatted_naive_date);
        }
        // this is Einspruch für Grundsteuermessbescheid
        if field.key == "question_n0DgP9" && letter.objection_subjects.contains(&"Grundsteuermessbetrag".to_string()) {
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

        if field.key == "question_3ENVAL_price"  || field.key == "question_3XpoOP_price" {
            invoice.payment.price = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_currency"  || field.key == "question_3XpoOP_currency" {
            invoice.payment.currency = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_name"  || field.key == "question_3XpoOP_name" {
            invoice.payment.name = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_email" || field.key == "question_3XpoOP_email" {
            invoice.payment.email = current_value[0].clone();
        }
        if field.key == "question_3ENVAL_link"  || field.key == "question_3XpoOP_link" {
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
            error!("sth. went wrong getting the tax office address: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Die angegebenen Finanzamtdaten waren vermutlich fehlerhaft. Das Finanzamt konnte nicht in unserer Datenbank gefunden werden.")));
        }
    };

    info!("tax office address: {:?}", tax_office_address_object);

    letter.receiver_office_name = tax_office_address_object.name;
    letter.receiver_office_zip = tax_office_address_object.zip;
    letter.receiver_office_city = tax_office_address_object.city;
    letter.receiver_office_address = format!("{} {}", tax_office_address_object.street, tax_office_address_object.number);

    debug!("start_now: {:?}", &meta_start_now);
    debug!("no_warranty: {:?}", &meta_no_warranty);
    debug!("origin_page: {:?}", &meta_origin_page);
    debug!("token: {:?}", &meta_token);

    if meta_start_now.value.parse() == Ok(false) || meta_no_revocation.value.parse() == Ok(false) {
        error!("Missing revocation acceptance.");
        return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Die Zustimmung zur Ausführung des Vertrags vor Ablauf der Widerrufsfrist und/oder den Verlust des Widerrufsrechts dadurch fehlt.")));
    }
    if meta_no_warranty.value.parse() == Ok(false) {
        error!("Missing warranty acceptance.");
        return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Es fehlt die Zustimmung zum Garantieausschluss.")));
    }
    if meta_origin_page.value != "/fragebogen.html" || meta_token.value != APP_TOKEN {
        error!("There might have been a tempered call from origin {} and with token {}.", meta_origin_page.value, meta_token.value);
        return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Der Aufruf war fehlerhaft!")));
    }

    let mapping_hash = get_mapping_hash(payload.data.respondent_id.as_str(), payload.data.submission_id.as_str(), letter.email.as_str());

    let mapping_base_path = match create_path(mapping_hash.to_string(), MAPPING_PATH.to_string()) {
        Ok(path) => path,
        Err(e) => {
            error!("Brieferstellung, path creation: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (path).")));
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
            error!("Mapping redirect, template rendering: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (redirect_render).")));
        }
    };
    match fs::write(mapping_path, mapping_result) {
        Ok(_) => {},
        Err(e) => {
            error!("Mapping redirect creation failed: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (redirect_mapping).")));
        }
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
            error!("Brieferstellung, context serializing: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (context).")));
        }
    };

    let letter_result = match TEMPLATES.render(TEMPLATE_NAME_LETTER, &letter_context) {
        Ok(result) => result,
        Err(e) => {
            error!("Brieferstellung, template rendering: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (render).")));
        }
    };

    match fs::write(letter_path, letter_result) {
        Ok(_) => {},
        Err(_) => {
            error!("Etwas ging schief beim Erstellen des Briefs (3).");
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen des Briefs (write).")));
        }
    }

    let invoice_context = match Context::from_serialize(&invoice) {
        Ok(result) => result,
        Err(e) => {
            error!("Rechnung, context serializing: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Rechnung (context).")));
        }
    };

    let invoice_result = match TEMPLATES.render(TEMPLATE_NAME_INVOICE, &invoice_context) {
        Ok(result) => result,
        Err(e) => {
            error!("Rechnung, template rendering: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Rechnung (render).")));
        }
    };

    match fs::write(invoice_path.clone(), invoice_result) {
        Ok(_) => {},
        Err(_) => {
            error!("Etwas ging schief beim Erstellen der Rechnung (3).");
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Rechnung (write).")));
        }
    }

    let index_context = match Context::from_serialize(&index) {
        Ok(result) => result,
        Err(e) => {
            error!("Index, context serializing: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Übersicht (context).")));
        }
    };

    let index_result = match TEMPLATES.render(TEMPLATE_NAME_INDEX, &index_context) {
        Ok(result) => result,
        Err(e) => {
            error!("Index, template rendering: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Übersicht (render).")));
        }
    };

    match fs::write(index_path, index_result) {
        Ok(_) => {},
        Err(e) => {
            error!("Etwas ging schief beim Erstellen der Übersicht (3): {}.", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Übersicht (write).")));
        }
    }

    let list_context = match Context::from_serialize(&list) {
        Ok(result) => result,
        Err(e) => {
            error!("List, context serializing: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Tipps (context).")));
        }
    };

    let list_result = match TEMPLATES.render(TEMPLATE_NAME_LIST, &list_context) {
        Ok(result) => result,
        Err(e) => {
            error!("List, template rendering: {}", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Tipps (render).")));
        }
    };

    match fs::write(list_path, list_result) {
        Ok(_) => {},
        Err(e) => {
            error!("Etwas ging schief beim Erstellen der Tipps (3): {}.", e);
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas lief schief beim Erstellen der Tipps (write).")));
        }
    }

    let sleep_time = time::Duration::from_millis(1600);
    thread::sleep(sleep_time);

    let _pdf_creation_result = match create_pdf_by_id(base_path.clone()) {
        Some(_) => "success".to_string(),
        None => {
            error!("PDF creation failed for unknown reason");
            return Err((StatusCode::BAD_REQUEST, headers, get_error_page("Etwas ging schief beim Erstellen des PDFs.")));
        },
    };

    if !pay_after {
        match send_email(&letter, link, get_email_config(state.email_user.clone(), state.email_pass.clone())) {
            // ok is just fine
            Ok(_) => {},
            Err(e) => error!("unexpected error while sending email: {}", e)
        };
    }

    let pdf_invoice_path = format!("{}/{}", base_path, RESULT_NAME_INVOICE);
    let invoice_file_name = format!("{}-Rechnung-{}_{}", date_now, invoice.first_name, invoice.last_name);

    match send_email_owner(pdf_invoice_path, invoice_file_name, get_email_config(state.email_user.clone(), state.email_pass.clone())).await {
        // ok is just fine
        Ok(_) => {},
        Err(e) => error!("unexpected error while sending invoice email home: {}", e)
    };

    return Ok((StatusCode::CREATED, headers, "ok".to_string()));
}

pub async fn get_html(Path(params): Path<HashMap<String, String>>) -> impl IntoResponse {
    let headers = [(header::CONTENT_TYPE, "text/html")];

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
        warn!("trying to get page of type {} which doesn't exist.", &page);
        return Err((StatusCode::NOT_FOUND, headers, get_error_page("Diese Seite existiert nicht.")));
    }

    let page_result = get_html_page(name, page, TARGET_PATH);

    return match page_result {
        Ok(result) => Ok((StatusCode::OK, headers, result)),
        Err(e) => {
            error!("couldn't get page with name {}: {}", &name, e);
            return Err((StatusCode::NOT_FOUND, headers, get_error_page("Diese Seite existiert nicht.")));
        }
    }
}

pub async fn get_result_page(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let headers = [(header::CONTENT_TYPE, "text/html")];

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

    let page_result = get_html_page(name, &format!("{}/index", page.clone()), TARGET_PATH);

    return match page_result {
        Ok(result) => Ok((StatusCode::OK, headers, result)),
        Err(e) => {
            info!("couldn't get page with name {}: {}", &name, e);
            return Err((StatusCode::NOT_FOUND, headers, get_error_page("Diese Seite existiert nicht.")));
        }
    }
}

pub async fn hello() -> &'static str {
    "v1"
}

pub async fn get_pdf(Path(params): Path<HashMap<String, String>>) -> impl IntoResponse {
    let headers = [(header::CONTENT_TYPE, "text/html")];

    let name = match params.get("id") {
        None => "",
        Some(val) => val.as_str(),
    };
    let page = match params.get("type") {
        None => "",
        Some(val) => val.as_str(),
    };

    let allowed_types = vec!["letter", "invoice", "list"];

    if !allowed_types.contains(&page) {
        info!("trying to get page of type {} which doesn't exist.", &page);
        return Err((StatusCode::NOT_FOUND, headers, get_error_page("Etwas lief schief beim Erstellen des Downloads.")));
    }

    let pdf_name = match page {
        "letter" => RESULT_NAME_LETTER,
        "invoice" => RESULT_NAME_INVOICE,
        "list" => RESULT_NAME_LIST,
        _ => "",
    };

    let file_path = format!("{}/{}/{}", TARGET_PATH, name, pdf_name);

    let file = match File::open(file_path).await {
        Ok(file) => file,
        Err(err) => {
            info!("could not find pdf file with page {}, page {} and id {}, current dir: {}. With error: {}", &page, &pdf_name, &name, env::current_dir().unwrap().display(), err);
            return Err((StatusCode::NOT_FOUND, headers, get_error_page("Die Datei konnte nicht gefunden werden.")));
        },
    };
    
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/pdf".to_string()),
        (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", pdf_name)),
    ];

    Ok((headers, body))
}