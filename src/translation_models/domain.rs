use std::collections::HashMap;
use crate::translation_models::{Domain, TranslationError};

/// מודל זיהוי תחום
pub struct DomainModel {
    /// מילון מונחים לכל תחום
    domain_terms: HashMap<Domain, Vec<String>>,
    /// משקולות לכל תחום
    domain_weights: HashMap<Domain, f64>,
}

impl DomainModel {
    pub fn new() -> Self {
        let mut model = Self {
            domain_terms: HashMap::new(),
            domain_weights: HashMap::new(),
        };
        
        // אתחול מונחים טכניים
        model.domain_terms.insert(Domain::Technical, vec![
            "מערכת".to_string(),
            "התקנה".to_string(),
            "צינור".to_string(),
            "משאבה".to_string(),
            "לחץ".to_string(),
            "ספיקה".to_string(),
            "מגוף".to_string(),
            "ברז".to_string(),
            "מתזים".to_string(),
            "ספרינקלרים".to_string(),
        ]);
        
        // אתחול מונחים משפטיים
        model.domain_terms.insert(Domain::Legal, vec![
            "חוזה".to_string(),
            "תקנה".to_string(),
            "תקן".to_string(),
            "אישור".to_string(),
            "רישיון".to_string(),
            "הסכם".to_string(),
            "התחייבות".to_string(),
            "אחריות".to_string(),
        ]);
        
        // אתחול משקולות
        model.domain_weights.insert(Domain::Technical, 1.0);
        model.domain_weights.insert(Domain::Legal, 0.8);
        model.domain_weights.insert(Domain::General, 0.6);
        
        model
    }

    /// מזהה תחום
    pub fn detect(&self, text: &str) -> Result<Domain, TranslationError> {
        let mut scores = HashMap::new();
        
        // חישוב ציון לכל תחום
        for (domain, terms) in &self.domain_terms {
            let score = self.calculate_domain_score(text, terms);
            scores.insert(domain, score);
        }
        
        // בחירת התחום עם הציון הגבוה ביותר
        let domain = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(domain, _)| *domain)
            .unwrap_or(&Domain::General);
            
        Ok(domain.clone())
    }

    /// מחשב ציון לתחום
    fn calculate_domain_score(&self, text: &str, terms: &[String]) -> f64 {
        let mut score = 0.0;
        let text = text.to_lowercase();
        
        // חישוב מספר המופעים של כל מונח
        for term in terms {
            let term = term.to_lowercase();
            let count = text.matches(&term).count();
            score += count as f64;
        }
        
        // נרמול הציון לפי אורך הטקסט
        let words = text.split_whitespace().count();
        if words > 0 {
            score /= words as f64;
        }
        
        // החלת משקולות
        if let Some(weight) = self.domain_weights.get(&Domain::Technical) {
            score *= weight;
        }
        
        score
    }

    /// מוסיף מונח לתחום
    pub fn add_term(&mut self, domain: Domain, term: String) {
        if let Some(terms) = self.domain_terms.get_mut(&domain) {
            terms.push(term);
        }
    }

    /// מעדכן משקולת לתחום
    pub fn update_weight(&mut self, domain: Domain, weight: f64) {
        self.domain_weights.insert(domain, weight);
    }

    /// מאמן את המודל
    pub fn train(&mut self, texts: &[(String, Domain)]) -> Result<(), TranslationError> {
        for (text, domain) in texts {
            // עדכון משקולות על בסיס טקסט מאומת
            let score = self.calculate_domain_score(text, 
                self.domain_terms.get(&domain).unwrap());
                
            // עדכון משקולת התחום
            let current_weight = self.domain_weights.get(&domain).unwrap_or(&1.0);
            let new_weight = (current_weight + score) / 2.0;
            self.domain_weights.insert(domain.clone(), new_weight);
        }
        
        Ok(())
    }
} 