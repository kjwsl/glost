#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    Afrikaans,
    Dutch,
    English,
    French,
    German,
    Italian,
    Japanese,
    Korean,
    Mandarin,
    Portuguese,
    Russian,
    Spanish,
    Swedish,
}

impl Language {
    pub fn to_lang_code(&self) -> &str {
        match self {
            Language::Afrikaans => "af",
            Language::Dutch => "nl",
            Language::English => "en",
            Language::French => "fr",
            Language::German => "de",
            Language::Italian => "it",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Mandarin => "zh",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Spanish => "es",
            Language::Swedish => "sv",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Language::Afrikaans => "Afrikaans",
            Language::Dutch => "Dutch",
            Language::English => "English",
            Language::French => "French",
            Language::German => "German",
            Language::Italian => "Italian",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Mandarin => "Mandarin",
            Language::Portuguese => "Portuguese",
            Language::Russian => "Russian",
            Language::Spanish => "Spanish",
            Language::Swedish => "Swedish",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for Language {
    type Err = std::fmt::Error;
    fn from_str(s: &str) -> Result<Self, std::fmt::Error> {
        match s.to_lowercase().as_str() {
            "afrikaans" => Ok(Language::Afrikaans),
            "dutch" => Ok(Language::Dutch),
            "english" => Ok(Language::English),
            "french" => Ok(Language::French),
            "german" => Ok(Language::German),
            "italian" => Ok(Language::Italian),
            "japanese" => Ok(Language::Japanese),
            "korean" => Ok(Language::Korean),
            "mandarin" => Ok(Language::Mandarin),
            "portuguese" => Ok(Language::Portuguese),
            "russian" => Ok(Language::Russian),
            "spanish" => Ok(Language::Spanish),
            "swedish" => Ok(Language::Swedish),
            _ => Err(std::fmt::Error),
        }
    }
}

