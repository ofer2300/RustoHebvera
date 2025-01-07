use std::sync::Arc;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum MorphologyError {
    #[error("שגיאת ניתוח מורפולוגי: {0}")]
    AnalysisError(String),
    #[error("שגיאת ניתוח שורש: {0}")]
    RootAnalysisError(String),
    #[error("שגיאת ניתוח נטייה: {0}")]
    InflectionError(String),
    #[error("שגיאת ניתוח הקשר: {0}")]
    ContextError(String),
}

/// מין דקדוקי
#[derive(Debug, Clone, PartialEq)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// מספר דקדוקי
#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Singular,
    Plural,
    Dual,
}

/// מקרה דקדוקי (עבור רוסית)
#[derive(Debug, Clone, PartialEq)]
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Prepositional,
}

/// בניין (עבור עברית)
#[derive(Debug, Clone, PartialEq)]
pub enum HebrewBinyan {
    Paal,
    Piel,
    Hifil,
    Hitpael,
    Nifal,
    Pual,
    Hufal,
}

/// משקל (עבור עברית)
#[derive(Debug, Clone)]
pub struct HebrewPattern {
    pub pattern: String,
    pub description: String,
}

/// תוצאת ניתוח מורפולוגי בעברית
#[derive(Debug, Clone)]
pub struct HebrewMorphology {
    pub root: Vec<char>,
    pub pattern: Option<HebrewPattern>,
    pub binyan: Option<HebrewBinyan>,
    pub gender: Option<Gender>,
    pub number: Option<Number>,
}

/// תוצאת ניתוח מורפולוגי ברוסית
#[derive(Debug, Clone)]
pub struct RussianMorphology {
    pub base_form: String,
    pub gender: Option<Gender>,
    pub number: Option<Number>,
    pub case: Option<Case>,
}

/// מאפייני זמן (עבור פעלים)
#[derive(Debug, Clone, PartialEq)]
pub enum Tense {
    Past,
    Present,
    Future,
    Imperative,
    Infinitive,
}

/// מאפייני גוף
#[derive(Debug, Clone, PartialEq)]
pub enum Person {
    First,
    Second,
    Third,
}

/// מבנה מורכב לניתוח שורשים
#[derive(Debug, Clone)]
pub struct RootAnalysis {
    pub root_letters: Vec<char>,
    pub variations: Vec<String>,
    pub common_patterns: Vec<HebrewPattern>,
    pub frequency_score: f32,
}

/// מידע סמנטי נוסף
#[derive(Debug, Clone)]
pub struct SemanticInfo {
    pub domain: Vec<String>,
    pub register: String,
    pub usage_examples: Vec<String>,
}

/// תוצאת ניתוח מורפולוגי מורחבת בעברית
#[derive(Debug, Clone)]
pub struct EnhancedHebrewMorphology {
    pub basic: HebrewMorphology,
    pub root_analysis: Option<RootAnalysis>,
    pub tense: Option<Tense>,
    pub person: Option<Person>,
    pub is_construct_state: bool,
    pub semantic_info: Option<SemanticInfo>,
    pub confidence_score: f32,
}

/// תוצאת ניתוח מורפולוגי מורחבת ברוסית
#[derive(Debug, Clone)]
pub struct EnhancedRussianMorphology {
    pub basic: RussianMorphology,
    pub aspect: String,
    pub is_animate: bool,
    pub stress_position: Option<usize>,
    pub semantic_info: Option<SemanticInfo>,
    pub confidence_score: f32,
}

/// ממשק מורחב למנתח מורפולוגי
pub trait AdvancedMorphologicalAnalyzer: Send + Sync {
    fn analyze_hebrew_enhanced(&self, word: &str, context: Option<&str>) -> Result<EnhancedHebrewMorphology, MorphologyError>;
    fn analyze_russian_enhanced(&self, word: &str, context: Option<&str>) -> Result<EnhancedRussianMorphology, MorphologyError>;
    fn get_root_variations(&self, root: &[char]) -> Vec<String>;
    fn get_pattern_examples(&self, pattern: &HebrewPattern) -> Vec<String>;
    fn calculate_confidence(&self, analysis: &EnhancedHebrewMorphology) -> f32;
}

pub mod hebrew;
pub mod russian;
pub mod cache;
pub mod utils;
pub mod semantic;
pub mod patterns;
pub mod statistics; 