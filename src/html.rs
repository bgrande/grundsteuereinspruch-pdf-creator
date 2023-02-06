use anyhow::Result as AnyResult;
use std::fs;

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

fn get_target_path(id: String, target_path: String) -> AnyResult<String> {
    let html_path = format!("{}/{}", target_path, id);
    Ok(html_path)
}

