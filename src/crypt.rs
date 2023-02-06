use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use anyhow::Result as AnyResult;
use chrono::{Utc};
use rand::{Rng};
use ring::digest::{Context as RingContext, SHA256};


#[derive(Hash)]
struct ToHash {
    value: String,
}

pub fn create_random_id() -> AnyResult<String> {
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

pub fn get_mapping_hash(respondent: &str, submission: &str, email: &str) -> u64 {
    let to_hash = ToHash {
        value: format!("{}{}{}", submission, respondent, email),
    };

    calculate_hash(to_hash)
}

pub fn generate_customer_id(first_name: &String, last_name: &String, email: &String) -> String {
    let hash_object = ToHash {
        value: format!("{}{}{}{}", Utc::now().format("%Y%m%dH%M%S"), first_name, last_name, email),
    };

    let hashed = calculate_hash(hash_object).to_string();

    return format!(
        "{}",
        hashed.split_at(7).0
    )
}

pub fn generate_invoice_id(customer_id: &String) -> String {
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

fn calculate_hash(to_hash: ToHash) -> u64 {
    let mut hasher = DefaultHasher::new();
    to_hash.hash(&mut hasher);
    hasher.finish()
}