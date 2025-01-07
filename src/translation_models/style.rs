use std::collections::HashMap;
use crate::translation_models::{Style, TranslationError};

/// מודל זיהוי סגנון
pub struct StyleModel {
    /// מאפייני סגנון
    style_features: HashMap<Style, StyleFeatures>,
    /// משקולות סגנון
    style_weights: HashMap<Style, f64>,
}

/// מאפייני סגנון
#[derive(Debug, Clone)]
pub struct StyleFeatures {
    /// מילים אופייניות
    characteristic_words: Vec<String>,
    /// תבניות תחביריות
    syntax_patterns: Vec<String>,
    /// רמת פורמליות
    formality_level: f64,
}

impl StyleModel {
    pub fn new() -> Self {
        let mut model = Self {
            style_features: HashMap::new(),
            style_weights: HashMap::new(),
        };
        
        // אתחול מאפייני סגנון פורמלי
        model.style_features.insert(Style::Formal, StyleFeatures {
            characteristic_words: vec![
                "להלן".to_string(),
                "בהתאם".to_string(),
                "לפיכך".to_string(),
                "כדלקמן".to_string(),
                "באמצעות".to_string(),
                "בהתייחס".to_string(),
            ],
            syntax_patterns: vec![
                "יש לציין כי".to_string(),
                "ניתן לקבוע כי".to_string(),
                "בהתאם לאמור".to_string(),
            ],
            formality_level: 0.9,
        });
        
        // אתחול מאפייני סגנון מקצועי
        model.style_features.insert(Style::Professional, StyleFeatures {
            characteristic_words: vec![
                "מערכת".to_string(),
                "מפרט".to_string(),
                "תקן".to_string(),
                "נתונים".to_string(),
                "ביצועים".to_string(),
                "יעילות".to_string(),
            ],
            syntax_patterns: vec![
                "בהתאם למפרט".to_string(),
                "על פי התקן".to_string(),
                "בהתאם לדרישות".to_string(),
            ],
            formality_level: 0.7,
        });
        
        // אתחול מאפייני סגנון יומיומי
        model.style_features.insert(Style::Casual, StyleFeatures {
            characteristic_words: vec![
                "בערך".to_string(),
                "בסדר".to_string(),
                "פשוט".to_string(),
                "רגיל".to_string(),
                "כזה".to_string(),
                "ככה".to_string(),
            ],
            syntax_patterns: vec![
                "אפשר גם".to_string(),
                "זה בסדר".to_string(),
                "פשוט צריך".to_string(),
            ],
            formality_level: 0.3,
        });
        
        // אתחול משקולות
        model.style_weights.insert(Style::Formal, 1.0);
        model.style_weights.insert(Style::Professional, 0.8);
        model.style_weights.insert(Style::Casual, 0.6);
        
        model
    }

    /// מזהה סגנון
    pub fn detect(&self, text: &str) -> Result<Style, TranslationError> {
        let mut scores = HashMap::new();
        
        // חישוב ציון לכל סגנון
        for (style, features) in &self.style_features {
            let score = self.calculate_style_score(text, features);
            scores.insert(style, score);
        }
        
        // בחירת הסגנון עם הציון הגבוה ביותר
        let style = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(style, _)| *style)
            .unwrap_or(&Style::Casual);
            
        Ok(style.clone())
    }

    /// מחשב ציון סגנון
    fn calculate_style_score(&self, text: &str, features: &StyleFeatures) -> f64 {
        let mut score = 0.0;
        let text = text.to_lowercase();
        
        // חישוב ציון מילים אופייניות
        for word in &features.characteristic_words {
            let word = word.to_lowercase();
            let count = text.matches(&word).count();
            score += count as f64;
        }
        
        // חישוב ציון תבניות תחביריות
        for pattern in &features.syntax_patterns {
            let pattern = pattern.to_lowercase();
            let count = text.matches(&pattern).count();
            score += count as f64 * 2.0; // משקל כפול לתבניות
        }
        
        // נרמול הציון
        let words = text.split_whitespace().count();
        if words > 0 {
            score /= words as f64;
        }
        
        // שקלול רמת פורמליות
        score *= features.formality_level;
        
        score
    }

    /// מוסיף מאפיין סגנון
    pub fn add_feature(&mut self, style: Style, word: String, is_pattern: bool) {
        if let Some(features) = self.style_features.get_mut(&style) {
            if is_pattern {
                features.syntax_patterns.push(word);
            } else {
                features.characteristic_words.push(word);
            }
        }
    }

    /// מעדכן רמת פורמליות
    pub fn update_formality(&mut self, style: Style, level: f64) {
        if let Some(features) = self.style_features.get_mut(&style) {
            features.formality_level = level;
        }
    }

    /// מאמן את המודל
    pub fn train(&mut self, texts: &[(String, Style)]) -> Result<(), TranslationError> {
        for (text, style) in texts {
            // עדכון משקולות על בסיס טקסט מאומת
            if let Some(features) = self.style_features.get(&style) {
                let score = self.calculate_style_score(text, features);
                
                // עדכון משקולת הסגנון
                let current_weight = self.style_weights.get(&style).unwrap_or(&1.0);
                let new_weight = (current_weight + score) / 2.0;
                self.style_weights.insert(style.clone(), new_weight);
            }
        }
        
        Ok(())
    }
} 