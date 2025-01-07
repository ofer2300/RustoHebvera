mod domain;
mod style;

pub use domain::DomainModel;
pub use style::StyleModel;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// הקשר תרגום
#[derive(Debug, Clone)]
pub struct TranslationContext {
    /// תחום
    pub domain: Domain,
    /// סגנון
    pub style: Style,
    /// רמת פורמליות
    pub formality: Formality,
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
#[derive(Debug)]
pub enum TranslationError {
    InvalidInput(String),
    NetworkError(String),
    DatabaseError(String),
    QualityCheckFailed(String),
}

#[derive(Debug)]
pub struct QualityResult {
    pub check_name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug)]
pub struct TranslationCache {
    pub entries: std::collections::HashMap<String, String>,
}

impl TranslationCache {
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ContextAnalyzer;

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct TermAnalyzer;

impl TermAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct LearningModel;

impl LearningModel {
    pub fn new() -> Self {
        Self
    }
} 