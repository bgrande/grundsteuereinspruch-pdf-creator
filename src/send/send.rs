use anyhow::Result as AnyResult;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::fs;
use lettre::message::header::{ContentTransferEncoding, ContentType};
use log::{error, info};

use crate::objects::Letter;

const CONFIG_FILE: &str = "data/db/email.json";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EmailConfig {
    from_name: String,
    from_email: String,
    smtp_host: String,
    smtp_user: String,
    smtp_pass: String,
    smtp_port: String,
}

pub(crate) fn get_email_config(user: String, pass: String) -> AnyResult<EmailConfig> {
    let data_result = fs::read_to_string(CONFIG_FILE);
    let data = data_result?;
    let mut json_config: EmailConfig = serde_json::from_str(&data)?;

    json_config.smtp_user = user;
    json_config.smtp_pass = pass;

    Ok(json_config)
}

pub(crate) fn send_email(letter: &Letter, link: String, email_conf: AnyResult<EmailConfig>) -> AnyResult<bool> {
    let email = &letter.email;

    let body = format!(
"Hallo {} {}!\r\n\r\nSie haben soeben den Fragebogen zur Brieferstellung auf grundsteuereinspruch.online ausgefüllt.\r\n\r\n
Ihr Downloadlink lautet:\r\n\
{}.\r\n\r\n
Das Passwort ist Ihre Postleitzahl!\r\n\r\n
Bei Fragen antworten Sie einfach auf diese E-Mail!\r\n\r\n
Mit freundlichen Grüßen\r\n\
Ihr Grundsteuereinspruch Online Team\r\n\r\n\
www.grundsteuereinspruch.online
mail@grundsteuereinspruch.online
",
        letter.first_name,
        letter.last_name,
        link
    );

    let email_config = email_conf?;

    let from = format!("{} <{}>", email_config.from_name, email_config.from_email);
    let subject = "Ihr Brief von Grundsteuereinspruch Online";
    let to = format!("{} {} <{}>", letter.first_name, letter.last_name, email);

    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .header(ContentTransferEncoding::Base64)
        .header(ContentType::TEXT_PLAIN)
        .subject(subject)
        .body(body)
        ?;

    let creds = Credentials::new(email_config.smtp_user, email_config.smtp_pass);

    let mailer = SmtpTransport::relay(&email_config.smtp_host)?
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => {
            info!("Email sent successfully!");
        }
        Err(e) => {
            error!("Could not send email: {:?}", e);
        }
    }

    Ok(true)
}
