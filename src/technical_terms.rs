use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TechnicalTerm {
    pub hebrew: String,
    pub russian: String,
    pub english: Option<String>,
    pub context: String,
    pub standards: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct TermsDatabase {
    terms: HashMap<String, TechnicalTerm>,
    russian_to_hebrew: HashMap<String, String>,
}

impl TermsDatabase {
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
            russian_to_hebrew: HashMap::new(),
        }
    }

    pub fn add_term(&mut self, term: TechnicalTerm) {
        self.russian_to_hebrew.insert(term.russian.clone(), term.hebrew.clone());
        self.terms.insert(term.hebrew.clone(), term);
    }

    pub fn get_russian_translation(&self, hebrew_term: &str) -> Option<&String> {
        self.terms.get(hebrew_term).map(|term| &term.russian)
    }

    pub fn get_hebrew_translation(&self, russian_term: &str) -> Option<&String> {
        self.russian_to_hebrew.get(russian_term)
    }

    pub fn get_all_hebrew_terms(&self) -> impl Iterator<Item = &String> {
        self.terms.keys()
    }

    pub fn get_all_russian_terms(&self) -> impl Iterator<Item = &String> {
        self.russian_to_hebrew.keys()
    }

    pub fn get_term_context(&self, term: &str) -> Option<&String> {
        self.terms.get(term).map(|t| &t.context)
    }

    pub fn get_term_standards(&self, term: &str) -> Option<&Vec<String>> {
        self.terms.get(term).map(|t| &t.standards)
    }
}

// יצירת מונחים בסיסיים
pub fn create_initial_terms() -> TermsDatabase {
    let mut db = TermsDatabase::new();
    
    // הוספת מונחים בסיסיים
    db.add_term(TechnicalTerm {
        hebrew: "ראש ספרינקלר".to_string(),
        russian: "ороситель/спринклер".to_string(),
        english: Some("Sprinkler Head".to_string()),
        context: "מערכות כיבוי אש אוטומטיות".to_string(),
        standards: vec!["NFPA 13".to_string(), "תקן ישראלי 1596".to_string()],
        notes: None,
    });

    db.add_term(TechnicalTerm {
        hebrew: "צנרת אספקה".to_string(),
        russian: "питающий трубопровод".to_string(),
        english: Some("Supply Pipe".to_string()),
        context: "מערכות אינסטלציה".to_string(),
        standards: vec!["ГОСТ 3262-75".to_string()],
        notes: None,
    });

    db.add_term(TechnicalTerm {
        hebrew: "מגוף שליטה".to_string(),
        russian: "контрольно-сигнальный клапан".to_string(),
        english: Some("Control Valve".to_string()),
        context: "מערכות כיבוי אש".to_string(),
        standards: vec!["NFPA 13".to_string(), "ГОСТ 51052-2002".to_string()],
        notes: None,
    });

    db.add_term(TechnicalTerm {
        hebrew: "לחץ עבודה".to_string(),
        russian: "рабочее давление".to_string(),
        english: Some("Working Pressure".to_string()),
        context: "מערכות אינסטלציה וכיבוי אש".to_string(),
        standards: vec!["ГОСТ 356-80".to_string()],
        notes: None,
    });

    db
} 