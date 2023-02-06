use std::fs;

use anyhow::Result as AnyResult;
use chrono::{Locale, LocalResult, NaiveDateTime, Utc};
use chrono::LocalResult::Single;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::form::QuestionResult;
use crate::config::FORM_ID;
use crate::config::FORM_NAME;

#[derive(Serialize, Deserialize)]
pub struct Sender {
    pub first_name: String,
    pub last_name: String,
    pub company_name: String,
    pub street: String,
    pub number: String,
    pub zip: String,
    pub city: String,
    pub email: String,
}

pub fn get_date_from_micro(date: &str) -> Option<NaiveDateTime> {
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

pub fn is_valid_payload(payload: &QuestionResult) -> bool {
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

pub fn get_sender_object(base_path: &str, file_name: &str) -> AnyResult<Sender> {
    let file_path = format!("{}/{}", base_path, file_name);

    // todo: remove this logging
    info!("sender data file path: {}", file_path);

    let data = fs::read_to_string(file_path);
    let data_string = data?;
    let sender: Sender = serde_json::from_str(data_string.as_str())?;

    Ok(sender)
}

pub fn get_naive_date_from_string(date_value: String) -> AnyResult<NaiveDateTime, anyhow::Error> {
    Ok(NaiveDateTime::parse_from_str(
        format!("{}{}", date_value, "T00:05:00+00:00").as_str(),
        "%Y-%m-%dT%H:%M:%S%z"
    )?)
}

pub fn get_formatted_date_from_string(date: String, format: &str) -> AnyResult<String> {
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