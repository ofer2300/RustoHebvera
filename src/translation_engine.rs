use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::morphology::{
    HebrewMorphology, RussianMorphology,
    HebrewAnalyzer, RussianAnalyzer,
    MorphologyCache, MorphologyError,
};
use crate::neural::{NeuralTranslator, TranslatorConfig};
use crate::quality_control::QualityControl;
use crate::technical_terms::TermsDatabase;

pub struct TranslationEngine {
    hebrew_analyzer: Arc<HebrewAnalyzer>,
    russian_analyzer: Arc<RussianAnalyzer>,
    morphology_cache: Arc<MorphologyCache>,
    neural_translator: Arc<NeuralTranslator>,
    quality_control: Arc<QualityControl>,
    terms_db: Arc<TermsDatabase>,
}

impl TranslationEngine {
    pub async fn new() -> Result<Self> {
        let hebrew_analyzer = Arc::new(HebrewAnalyzer::new());
        let russian_analyzer = Arc::new(RussianAnalyzer::new());
        let morphology_cache = Arc::new(MorphologyCache::new(3600)); // TTL של שעה
        
        let config = TranslatorConfig {
            hidden_size: 256,
            embedding_dim: 128,
            num_layers: 2,
            num_heads: 8,
            dropout: 0.1,
            source_vocab_size: 50000,
            target_vocab_size: 50000,
        };
        
        let neural_translator = Arc::new(NeuralTranslator::new(
            config,
            Arc::new(Default::default()),
            Arc::new(Default::default()),
        )?);
        
        let quality_control = Arc::new(QualityControl::new());
        let terms_db = Arc::new(TermsDatabase::new());
        
        Ok(Self {
            hebrew_analyzer,
            russian_analyzer,
            morphology_cache,
            neural_translator,
            quality_control,
            terms_db,
        })
    }

    pub async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        // ניתוח מורפולוגי של טקסט המקור
        let morphological_analysis = match source_lang {
            "he" => self.analyze_hebrew(text).await?,
            "ru" => self.analyze_russian(text).await?,
            _ => return Err(anyhow::anyhow!("שפת מקור לא נתמכת")),
        };
        
        // תרגום בסיסי
        let base_translation = self.neural_translator.translate(&[text.to_string()])?
            .first()
            .ok_or_else(|| anyhow::anyhow!("שגיאה בתרגום"))?
            .clone();
        
        // התאמה דקדוקית של התרגום
        let grammar_adjusted = self.adjust_grammar(
            &base_translation,
            source_lang,
            target_lang,
            &morphological_analysis,
        ).await?;
        
        // בקרת איכות
        let validated = self.quality_control.validate_deep(&grammar_adjusted).await?;
        
        Ok(validated.text)
    }

    async fn analyze_hebrew(&self, text: &str) -> Result<HebrewMorphology> {
        // בדיקה במטמון
        if let Some(cached) = self.morphology_cache.get_hebrew(text) {
            return Ok(cached);
        }
        
        // ניתוח חדש
        let analysis = self.hebrew_analyzer.analyze(text)
            .map_err(|e| anyhow::anyhow!("שגיאת ניתוח מורפולוגי בעברית: {}", e))?;
        
        // שמירה במטמון
        self.morphology_cache.set_hebrew(text.to_string(), analysis.clone());
        
        Ok(analysis)
    }

    async fn analyze_russian(&self, text: &str) -> Result<RussianMorphology> {
        // בדיקה במטמון
        if let Some(cached) = self.morphology_cache.get_russian(text) {
            return Ok(cached);
        }
        
        // ניתוח חדש
        let analysis = self.russian_analyzer.analyze(text)
            .map_err(|e| anyhow::anyhow!("שגיאת ניתוח מורפולוגי ברוסית: {}", e))?;
        
        // שמירה במטמון
        self.morphology_cache.set_russian(text.to_string(), analysis.clone());
        
        Ok(analysis)
    }

    async fn adjust_grammar(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
        source_analysis: &(impl Into<HebrewMorphology> + Into<RussianMorphology>),
    ) -> Result<String> {
        let mut adjusted = text.to_string();
        
        match (source_lang, target_lang) {
            ("he", "ru") => {
                let hebrew_analysis: HebrewMorphology = source_analysis.clone().into();
                // התאמת מין ומספר
                if let Some(gender) = hebrew_analysis.gender {
                    adjusted = self.adjust_russian_gender(&adjusted, gender)?;
                }
                if let Some(number) = hebrew_analysis.number {
                    adjusted = self.adjust_russian_number(&adjusted, number)?;
                }
            }
            ("ru", "he") => {
                let russian_analysis: RussianMorphology = source_analysis.clone().into();
                // התאמת מין ומספר
                if let Some(gender) = russian_analysis.gender {
                    adjusted = self.adjust_hebrew_gender(&adjusted, gender)?;
                }
                if let Some(number) = russian_analysis.number {
                    adjusted = self.adjust_hebrew_number(&adjusted, number)?;
                }
            }
            _ => return Err(anyhow::anyhow!("צמד שפות לא נתמך")),
        }
        
        Ok(adjusted)
    }

    fn adjust_russian_gender(&self, text: &str, gender: Gender) -> Result<String> {
        // TODO: יישום התאמת מין ברוסית
        Ok(text.to_string())
    }

    fn adjust_russian_number(&self, text: &str, number: Number) -> Result<String> {
        // TODO: יישום התאמת מספר ברוסית
        Ok(text.to_string())
    }

    fn adjust_hebrew_gender(&self, text: &str, gender: Gender) -> Result<String> {
        // TODO: יישום התאמת מין בעברית
        Ok(text.to_string())
    }

    fn adjust_hebrew_number(&self, text: &str, number: Number) -> Result<String> {
        // TODO: יישום התאמת מספר בעברית
        Ok(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_translation_with_morphology() {
        let engine = TranslationEngine::new().await.unwrap();
        
        // בדיקת תרגום מעברית לרוסית
        let result = engine.translate("הספר הזה", "he", "ru").await;
        assert!(result.is_ok());
        
        // בדיקת תרגום מרוסית לעברית
        let result = engine.translate("эта книга", "ru", "he").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_morphological_analysis() {
        let engine = TranslationEngine::new().await.unwrap();
        
        // בדיקת ניתוח מורפולוגי בעברית
        let result = engine.analyze_hebrew("ספרים").await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.number, Some(Number::Plural));
        
        // בדיקת ניתוח מורפולוגי ברוסית
        let result = engine.analyze_russian("книги").await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.number, Some(Number::Plural));
    }

    #[tokio::test]
    async fn test_grammar_adjustment() {
        let engine = TranslationEngine::new().await.unwrap();
        
        // בדיקת התאמה דקדוקית מעברית לרוסית
        let hebrew_analysis = engine.analyze_hebrew("ספר גדול").await.unwrap();
        let result = engine.adjust_grammar(
            "большая книга",
            "he",
            "ru",
            &hebrew_analysis,
        ).await;
        assert!(result.is_ok());
        
        // בדיקת התאמה דקדוקית מרוסית לעברית
        let russian_analysis = engine.analyze_russian("большая книга").await.unwrap();
        let result = engine.adjust_grammar(
            "ספר גדול",
            "ru",
            "he",
            &russian_analysis,
        ).await;
        assert!(result.is_ok());
    }
} 