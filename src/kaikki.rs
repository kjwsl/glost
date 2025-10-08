use serde::{Deserialize, Serialize};

// Top-level struct for the JSON object
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub word: String,
    pub lang: String,
    pub lang_code: String,
    pub pos: String,
    pub senses: Vec<Sense>,
    pub head_templates: Option<Vec<HeadTemplate>>,
    pub categories: Option<Vec<String>>,
    pub sounds: Option<Vec<Sound>>,
}

// Struct for the "senses" array
#[derive(Serialize, Deserialize, Debug)]
pub struct Sense {
    pub links: Option<Vec<Vec<String>>>,
    pub form_of: Option<Vec<FormOf>>,
    pub glosses: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub raw_glosses: Option<Vec<String>>,
    pub examples: Option<Vec<Example>>,
}

// Struct for the "examples" array
#[derive(Serialize, Deserialize, Debug)]
pub struct Example {
    pub text: Option<String>,
    pub bold_text_offsets: Option<Vec<Vec<u32>>>,
    pub translation: Option<String>,
    pub english: Option<String>,
    pub bold_translation_offsets: Option<Vec<Vec<u32>>>,
    pub type_: Option<String>,
    pub raw_tags: Option<Vec<String>>,
}

// Struct for the "form_of" array
#[derive(Serialize, Deserialize, Debug)]
pub struct FormOf {
    pub word: String,
}

// Struct for the "head_templates" array
#[derive(Serialize, Deserialize, Debug)]
pub struct HeadTemplate {
    pub name: String,
    pub args: Option<HeadTemplateArgs>,
    pub expansion: Option<String>,
}

// Struct for the "args" object in head_templates
#[derive(Serialize, Deserialize, Debug)]
pub struct HeadTemplateArgs {
    #[serde(rename = "1")]
    pub lang: Option<String>,
    #[serde(rename = "2")]
    pub form: Option<String>,
}

// Struct for the "sounds" array
#[derive(Serialize, Deserialize, Debug)]
pub struct Sound {
    pub ipa: Option<String>,
    pub audio: Option<String>,
    pub ogg_url: Option<String>,
    pub mp3_url: Option<String>,
    pub rhymes: Option<String>,
}
