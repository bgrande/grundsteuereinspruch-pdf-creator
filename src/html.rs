use anyhow::Result as AnyResult;
use std::fs;
use log::error;
use tera::Context;
use crate::config::TEMPLATE_NAME_ERROR;
use crate::crypt::generate_error_id;
use crate::routes::TEMPLATES;

pub fn get_html_page(name: &str, page: &str, target_path: &str) -> AnyResult<String> {
    let file_path = format!("{}/{}/{}.html", target_path, name, page);
    let data = fs::read_to_string(file_path);
    let data_string = data?;

    Ok(data_string.clone())
}

pub fn create_path(file_id: String, target_path: String) -> AnyResult<String> {
    let path = get_target_path(file_id, target_path)?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn get_error_page(error_msg: &str) -> String {
    let error_id = generate_error_id(&error_msg);

    let mut error_context = Context::new();
    error_context.insert("error_id", &error_id);
    error_context.insert("error_msg", &error_msg);

    let error_result = match TEMPLATES.render(TEMPLATE_NAME_ERROR, &error_context) {
        Ok(result) => result,
        Err(e) => {
            error!("Error page, template rendering for id: {}: {}", error_id, e);
            format!("Fehler beim Erstellen der Fehlerseite. Fehler: {} mit id {}", e, error_id)
        }
    };

    error!("error message with id {} sent", error_id);

    // todo also send an E-Mail about the fail - helps with debugging

    error_result
}

fn get_target_path(id: String, target_path: String) -> AnyResult<String> {
    let html_path = format!("{}/{}", target_path, id);
    Ok(html_path)
}
