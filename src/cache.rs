use crate::Language;
use crate::ai::ExpressionAnalysis;
use crate::kaikki;
use rusqlite::{Connection, params};
use std::error::Error;
use std::sync::Mutex;

pub struct Cache {
    conn: Mutex<Connection>,
}

impl Cache {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let conn = Connection::open(path)?;

        // Initialize tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_cache (
                expression TEXT,
                context TEXT,
                lang TEXT,
                model TEXT,
                lemma TEXT,
                meaning TEXT,
                cefr TEXT,
                grammar TEXT,
                PRIMARY KEY (expression, context, lang, model)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS kaikki_cache (
                expression TEXT PRIMARY KEY,
                json_data TEXT
            )",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_ai_analysis(
        &self,
        expression: &str,
        context: &str,
        lang: Language,
        model: &str,
    ) -> Result<Option<ExpressionAnalysis>, Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT lemma, meaning, cefr, grammar FROM ai_cache WHERE expression = ? AND context = ? AND lang = ? AND model = ?"
        )?;

        let mut rows = stmt.query(params![expression, context, lang.to_string(), model])?;

        if let Some(row) = rows.next()? {
            Ok(Some(ExpressionAnalysis {
                lemma: row.get(0)?,
                meaning: row.get(1)?,
                cefr: row.get(2)?,
                grammar: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert_ai_analysis(
        &self,
        expression: &str,
        context: &str,
        lang: Language,
        model: &str,
        analysis: &ExpressionAnalysis,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO ai_cache (expression, context, lang, model, lemma, meaning, cefr, grammar) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                expression,
                context,
                lang.to_string(),
                model,
                analysis.lemma,
                analysis.meaning,
                analysis.cefr,
                analysis.grammar
            ],
        )?;
        Ok(())
    }

    pub fn get_kaikki_entries(
        &self,
        expression: &str,
    ) -> Result<Option<Vec<kaikki::Entry>>, Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT json_data FROM kaikki_cache WHERE expression = ?")?;
        let mut rows = stmt.query(params![expression])?;

        if let Some(row) = rows.next()? {
            let json_data: String = row.get(0)?;
            let entries: Vec<kaikki::Entry> = serde_json::from_str(&json_data)?;
            Ok(Some(entries))
        } else {
            Ok(None)
        }
    }

    pub fn insert_kaikki_entries(
        &self,
        expression: &str,
        entries: &[kaikki::Entry],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let json_data =
            serde_json::from_str::<serde_json::Value>(&serde_json::to_string(entries)?)?
                .to_string();
        conn.execute(
            "INSERT OR REPLACE INTO kaikki_cache (expression, json_data) VALUES (?, ?)",
            params![expression, json_data],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_ai_analysis() -> Result<(), Box<dyn Error + Send + Sync>> {
        let tmp = NamedTempFile::new()?;
        let cache = Cache::new(tmp.path().to_str().unwrap())?;

        let analysis = ExpressionAnalysis {
            lemma: "talo".to_string(),
            meaning: "house".to_string(),
            cefr: Some("A1".to_string()),
            grammar: Some("nominative".to_string()),
        };

        cache.insert_ai_analysis(
            "taloa",
            "Tämä on taloa.",
            Language::Finnish,
            "model",
            &analysis,
        )?;

        let cached =
            cache.get_ai_analysis("taloa", "Tämä on taloa.", Language::Finnish, "model")?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().lemma, "talo");

        let missing = cache.get_ai_analysis("missing", "context", Language::Finnish, "model")?;
        assert!(missing.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_kaikki_entries() -> Result<(), Box<dyn Error + Send + Sync>> {
        let tmp = NamedTempFile::new()?;
        let cache = Cache::new(tmp.path().to_str().unwrap())?;

        let entry = kaikki::Entry {
            word: "talo".to_string(),
            pos: "noun".to_string(),
            lang: "Finnish".to_string(),
            lang_code: "fi".to_string(),
            senses: vec![],
            categories: None,
            head_templates: None,
            sounds: None,
        };

        cache.insert_kaikki_entries("talo", &[entry.clone()])?;

        let cached = cache.get_kaikki_entries("talo")?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()[0].word, "talo");

        Ok(())
    }
}
