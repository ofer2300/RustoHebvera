use std::fmt;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Masculine,
    Feminine,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Number {
    Singular,
    Plural,
    Dual,
}

#[derive(Debug, Error)]
pub enum MorphologyError {
    #[error("שגיאת ניתוח מורפולוגי: {0}")]
    AnalysisError(String),
    #[error("שגיאת נטמון: {0}")]
    CacheError(String),
    #[error("שגיאת תבנית: {0}")]
    PatternError(String),
}

pub trait MorphologyAnalyzer {
    fn analyze(&self, text: &str) -> Result<MorphologyAnalysis, MorphologyError>;
    fn calculate_confidence(&self, analysis: &MorphologyAnalysis) -> f32;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologyAnalysis {
    pub base_form: String,
    pub gender: Option<Gender>,
    pub number: Option<Number>,
    pub confidence: f32,
}

pub mod hebrew;
pub mod russian;
pub mod cache;
pub mod patterns;
pub mod semantic;
pub mod statistics;

pub use hebrew::HebrewAnalyzer;
pub use russian::RussianAnalyzer;
pub use cache::MorphologyCache; 