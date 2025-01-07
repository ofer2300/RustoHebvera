pub mod morphology;
pub mod neural;
pub mod translation_engine;
pub mod quality_control;
pub mod gui;
pub mod learning_manager;
pub mod technical_terms;
pub mod vocabulary;
pub mod translation_models;

pub use morphology::{
    HebrewAnalyzer, RussianAnalyzer,
    MorphologyCache, MorphologyError,
    Gender, Number,
};

pub use neural::{
    NeuralTranslator,
    attention::{MultiHeadAttention, AttentionConfig},
};

pub use translation_engine::TranslationEngine;
pub use quality_control::{QualityControl, IssueSeverity};
pub use learning_manager::{LearningManager, LearningEvent, LearningEventType, UserFeedback};
pub use technical_terms::TechnicalTermsManager;
pub use vocabulary::{Vocabulary, VocabularyError}; 