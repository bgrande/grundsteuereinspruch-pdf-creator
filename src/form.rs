use serde::{Deserialize, Deserializer, Serialize};

fn parse_value<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AnyType<'a> {
        Str(&'a str),
        String(String),
        U64(u64),
        Vec(Vec<String>),
        Bool(bool),
        None,
    }

    Ok(match AnyType::deserialize(deserializer)? {
        AnyType::Str(v) => Some(vec![v.to_string()]),
        AnyType::String(v) => Some(vec![v]),
        AnyType::U64(v) => Some(vec![v.to_string()]),
        AnyType::Vec(v) => Some(v),
        AnyType::Bool(v) => Some(vec![v.to_string()]),
        AnyType::None => None,
    })
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormFieldOption {
    pub id: String,
    pub text: String,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub form_type: String,
    #[serde(deserialize_with="parse_value")]
    pub value: Option<Vec<String>>,
    pub options: Option<Vec<FormFieldOption>>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionData {
    pub response_id: String,
    pub submission_id: String,
    pub respondent_id: String,
    pub form_id: String,
    pub form_name: String,
    pub created_at: String,
    pub fields: Vec<FormField>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionResult {
    pub event_id: String,
    pub event_type: String,
    pub created_at: String,
    pub data: QuestionData,
}

#[derive(Debug)]
pub struct FormFieldMeta {
    pub key: String, /// ref, originPage, token
    pub value: String,
}

pub fn get_value_from_option(options: &Option<Vec<FormFieldOption>>, vec_val: Vec<String>) -> Vec<String> {
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