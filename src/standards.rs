use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Standard {
    pub code: String,
    pub name_he: String,
    pub name_ru: String,
    pub country: String,
    pub description_he: Option<String>,
    pub description_ru: Option<String>,
    pub related_standards: Vec<String>,
}

#[derive(Debug)]
pub struct StandardsDatabase {
    standards: HashMap<String, Standard>,
}

impl StandardsDatabase {
    pub fn new() -> Self {
        Self {
            standards: HashMap::new(),
        }
    }

    pub fn add_standard(&mut self, standard: Standard) {
        self.standards.insert(standard.code.clone(), standard);
    }

    pub fn get_standard(&self, code: &str) -> Option<&Standard> {
        self.standards.get(code)
    }

    pub fn get_related_standards(&self, code: &str) -> Vec<&Standard> {
        if let Some(standard) = self.standards.get(code) {
            standard
                .related_standards
                .iter()
                .filter_map(|related_code| self.standards.get(related_code))
                .collect()
        } else {
            vec![]
        }
    }
}

// יצירת מסד נתוני תקנים ראשוני
pub fn create_initial_standards() -> StandardsDatabase {
    let mut db = StandardsDatabase::new();
    
    // הוספת תקנים בסיסיים
    db.add_standard(Standard {
        code: "IS1596".to_string(),
        name_he: "תקן ישראלי 1596 - מערכות מתזים".to_string(),
        name_ru: "Израильский стандарт 1596 - Спринклерные системы".to_string(),
        country: "ישראל".to_string(),
        description_he: Some("תקן למערכות כיבוי אש אוטומטיות (ספרינקלרים)".to_string()),
        description_ru: Some("Стандарт для автоматических систем пожаротушения (спринклеров)".to_string()),
        related_standards: vec!["NFPA13".to_string()],
    });

    db.add_standard(Standard {
        code: "GOST3262-75".to_string(),
        name_he: "תקן ГОСТ 3262-75 - צנרת פלדה".to_string(),
        name_ru: "ГОСТ 3262-75 - Стальные трубы".to_string(),
        country: "רוסיה".to_string(),
        description_he: Some("תקן לצנרת פלדה מגולוונת למערכות אינסטלציה".to_string()),
        description_ru: Some("Стандарт для оцинкованных стальных труб в системах водоснабжения".to_string()),
        related_standards: vec![],
    });

    db
} 