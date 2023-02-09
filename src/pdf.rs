use html2pdf::{run, CliOptions, Error as H2PError};
use structopt::StructOpt;
use tracing::info;

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