use crate::Language;
use crate::ai::WordAnalysis;
use crate::kaikki;
use rusqlite::{params, Connection};
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
                word TEXT,
                context TEXT,
                lang TEXT,
                model TEXT,
                lemma TEXT,
                cefr TEXT,
                grammar TEXT,
                PRIMARY KEY (word, context, lang, model)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS kaikki_cache (
                word TEXT PRIMARY KEY,
                json_data TEXT
            )",
            [],
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn get_ai_analysis(
        &self,
        word: &str,
        context: &str,
        lang: Language,
        model: &str,
    ) -> Result<Option<WordAnalysis>, Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT lemma, cefr, grammar FROM ai_cache WHERE word = ? AND context = ? AND lang = ? AND model = ?"
        )?;
        
        let mut rows = stmt.query(params![word, context, lang.to_string(), model])?;

        if let Some(row) = rows.next()? {
            Ok(Some(WordAnalysis {
                lemma: row.get(0)?,
                cefr: row.get(1)?,
                grammar: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert_ai_analysis(
        &self,
        word: &str,
        context: &str,
        lang: Language,
        model: &str,
        analysis: &WordAnalysis,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO ai_cache (word, context, lang, model, lemma, cefr, grammar) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                word,
                context,
                lang.to_string(),
                model,
                analysis.lemma,
                analysis.cefr,
                analysis.grammar
            ],
        )?;
        Ok(())
    }

    pub fn get_kaikki_entries(
        &self,
        word: &str,
    ) -> Result<Option<Vec<kaikki::Entry>>, Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT json_data FROM kaikki_cache WHERE word = ?")?;
        let mut rows = stmt.query(params![word])?;

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
        word: &str,
        entries: &[kaikki::Entry],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conn = self.conn.lock().unwrap();
        let json_data = serde_json::from_str::<serde_json::Value>(&serde_json::to_string(entries)?)?.to_string();
        conn.execute(
            "INSERT OR REPLACE INTO kaikki_cache (word, json_data) VALUES (?, ?)",
            params![word, json_data],
        )?;
        Ok(())
    }
}
