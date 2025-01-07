mod domain;
mod style;

pub use domain::DomainModel;
pub use style::StyleModel;

use chrono::{DateTime, Utc};
use thiserror::Error;
use std::collections::HashMap;

/// הקשר תרגום
#[derive(Debug, Clone)]
pub struct TranslationContext {
    /// תחום
    pub domain: Domain,
    /// סגנון
    pub style: Style,
    /// רמת פורמליות
    pub formality: Formality,
    pub metadata: HashMap<String, String>,
}

/// תחומי תרגום
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Domain {
    /// טכני
    Technical,
    /// משפטי
    Legal,
    /// כללי
    General,
    Medical,
    Custom(String),
}

/// סגנונות תרגום
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Style {
    /// פורמלי
    Formal,
    /// מקצועי
    Professional,
    /// יומיומי
    Casual,
    Informal,
    Technical,
    Custom(String),
}

/// רמות פורמליות
#[derive(Debug, Clone)]
pub enum Formality {
    /// גבוהה
    High,
    /// בינונית
    Medium,
    /// נמוכה
    Low,
    Custom(String),
}

/// מונח טכני
#[derive(Debug, Clone)]
pub struct TechnicalTerm {
    /// מזהה
    pub id: String,
    /// מונח מקור
    pub source: String,
    /// מונח יעד
    pub target: String,
    /// תחום
    pub domain: Domain,
    /// הערות
    pub notes: Option<String>,
    /// תאריך עדכון
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// רשומת תרגום
#[derive(Debug, Clone)]
pub struct TranslationRecord {
    /// טקסט מקור
    pub source: String,
    /// טקסט יעד
    pub target: String,
    /// תאריך
    pub timestamp: DateTime<Utc>,
    pub context: TranslationContext,
}

/// שגיאות תרגום
#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("שגיאת מודל: {0}")]
    ModelError(String),
    
    #[error("שגיאת אוצר מילים: {0}")]
    VocabularyError(String),
    
    #[error("שגיאת הקשר: {0}")]
    ContextError(String),
    
    #[error("שגיאת מונח טכני: {0}")]
    TechnicalTermError(String),
    
    #[error("שגיאת למידה: {0}")]
    LearningError(String),
    
    #[error("שגיאה כללית: {0}")]
    GeneralError(String),
}

#[derive(Debug)]
pub struct QualityResult {
    pub score: f64,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug)]
pub struct TranslationCache {
    pub source: String,
    pub target: String,
    pub context: TranslationContext,
    pub quality_score: f64,
}

impl TranslationCache {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            target: String::new(),
            context: TranslationContext {
                domain: Domain::Technical,
                style: Style::Formal,
                formality: Formality::High,
                metadata: HashMap::new(),
            },
            quality_score: 0.0,
        }
    }
}

pub trait ContextAnalyzer {
    fn analyze_context(&self, text: &str) -> Result<TranslationContext, TranslationError>;
    fn validate_context(&self, context: &TranslationContext) -> Result<bool, TranslationError>;
}

pub trait TermAnalyzer {
    fn extract_terms(&self, text: &str) -> Result<Vec<TechnicalTerm>, TranslationError>;
    fn validate_term(&self, term: &TechnicalTerm) -> Result<bool, TranslationError>;
}

pub trait LearningModel {
    fn train(&mut self, source: &str, target: &str, context: &TranslationContext) -> Result<(), TranslationError>;
    fn evaluate(&self, source: &str, target: &str) -> Result<f64, TranslationError>;
} 