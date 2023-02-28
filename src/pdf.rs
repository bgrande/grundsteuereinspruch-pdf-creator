use std::path::PathBuf;
use html2pdf::{run, Options, Error as H2PError};
use log::{error};

use crate::config::TEMPLATE_NAME_LETTER;
use crate::config::TEMPLATE_NAME_INVOICE;
use crate::config::TEMPLATE_NAME_LIST;

pub const RESULT_NAME_LETTER: &str = "Grundsteuereinspruch.pdf";
pub const RESULT_NAME_INVOICE: &str = "Grundsteuereinspruch-Rechnung.pdf";
pub const RESULT_NAME_LIST: &str = "Grundsteuereinspruch-Liste.pdf";

pub fn create_pdf_by_id(base_path: String) -> Option<bool> {
    let html2pdf_letter = match html2pdf(
        format!("{}/{}", base_path, TEMPLATE_NAME_LETTER),
        format!("{}/{}", base_path, RESULT_NAME_LETTER)
    ) {
        Ok(result) => result,
        Err(e) => {
            error!("sth. went wrong creating the pdf for the letter: {}", e);
            return None
        },
    };

    let html2pdf_invoice = match html2pdf(
        format!("{}/{}", base_path, TEMPLATE_NAME_INVOICE),
        format!("{}/{}", base_path, RESULT_NAME_INVOICE)
    ) {
        Ok(result) => result,
        Err(e) => {
            error!("sth. went wrong creating the pdf for the invoice: {}", e);
            return None
        },
    };

    let html2pdf_list = match html2pdf(
        format!("{}/{}", base_path, TEMPLATE_NAME_LIST),
        format!("{}/{}", base_path, RESULT_NAME_LIST)
    ) {
        Ok(result) => result,
        Err(e) => {
            error!("sth. went wrong creating the pdf for the tipps: {}", e);
            return None
        },
    };

    Some(to_result(html2pdf_letter) && to_result(html2pdf_invoice) && to_result(html2pdf_list))
}

fn html2pdf(path: String, result_path: String) -> std::result::Result<(), H2PError> {
    let input_path = PathBuf::from(path);
    let output_path = PathBuf::from(result_path);

    let options = Options {
        input: input_path,
        output: Some(output_path),
        landscape: false,
        background: false,
        wait: None,
        header: None,
        footer: None,
        paper: None,
        scale: None,
        range: None,
        margin: None,
    };

    // Let's go
    run(&options)
}

fn to_result(from_2pdf: ()) -> bool {
    return match from_2pdf {
        () => true,
    };
}