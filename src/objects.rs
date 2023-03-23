use serde::{Serialize};

#[derive(Serialize)]
pub struct Letter {
    pub first_name: String,
    pub last_name: String,
    pub street: String,
    pub number: String,
    pub zip: String,
    pub city: String,
    pub email: String,
    pub phone: String,
    pub max_sender_count: i8,
    pub reference_number: String,
    pub sender_names_intro: String,
    pub sender_names: Vec<String>,
    pub receiver_office_name: String,
    pub receiver_office_address: String,
    pub receiver_office_zip: String,
    pub receiver_office_city: String,
    pub objection_subjects: Vec<String>,
    pub objection_subject_start_dates: Vec<String>,
    pub objection_subject_reasons: Vec<String>,
    pub date_created: String,
    pub sent_date: String,
    pub subject_text: String,
    pub additional_senders: bool,
    pub multiple_objection_subjects: bool,
}
#[derive(Serialize)]
pub struct Payment {
    pub price: String,
    pub currency: String,
    pub name: String,
    pub email: String,
    pub link: String,
}
#[derive(Serialize)]
pub struct Invoice {
    pub sender_first_name: String,
    pub sender_last_name: String,
    pub sender_company_name: String,
    pub sender_street: String,
    pub sender_number: String,
    pub sender_zip: String,
    pub sender_city: String,
    pub sender_email: String,
    pub first_name: String,
    pub last_name: String,
    pub street: String,
    pub number: String,
    pub zip: String,
    pub city: String,
    pub email: String,
    pub date: String,
    pub invoice_id: String,
    pub customer_id: String,
    pub subject_text: String,
    pub payment: Payment,
    pub invoice_objects: Vec<String>,
}
#[derive(Serialize)]
pub struct Index {
    pub first_name: String,
    pub last_name: String,
    pub date: String,
    pub file_id: String,
    pub sent_date: String,
    pub deadline_date: String,
}
#[derive(Serialize)]
pub struct List {
    pub first_name: String,
    pub last_name: String,
    pub date: String,
    pub file_id: String,
    pub sent_date: String,
    pub deadline_date: String,
}
#[derive(Serialize)]
pub struct EMail {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub date: String,
    pub link: String,
    pub sent_date: String,
    pub deadline_date: String,
}

#[derive(Serialize)]
pub struct Mapping {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub mapping_id: String,
}