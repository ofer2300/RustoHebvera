use crate::translation_models::{TranslationContext, Domain, Style, TranslationError, QualityResult};

pub struct QualityControl {
    length_ratio_threshold: f64,
    formality_threshold: f64,
}

impl QualityControl {
    pub fn new() -> Self {
        Self {
            length_ratio_threshold: 1.5,
            formality_threshold: 0.8,
        }
    }

    pub fn check_quality(&self, source: &str, translation: &str, context: &TranslationContext) -> Result<Vec<QualityResult>, TranslationError> {
        let mut results = Vec::new();

        // בדיקת יחס אורך
        let length_ratio = translation.len() as f64 / source.len() as f64;
        if length_ratio > self.length_ratio_threshold {
            results.push(QualityResult {
                check_name: "length_ratio".to_string(),
                passed: false,
                message: format!("Length ratio ({:.2}) exceeds threshold ({:.2})", length_ratio, self.length_ratio_threshold),
            });
        }

        // בדיקת פורמליות
        if context.style == Style::Formal {
            let formality_score = self.calculate_formality_score(translation);
            if formality_score < self.formality_threshold {
                results.push(QualityResult {
                    check_name: "formality".to_string(),
                    passed: false,
                    message: format!("Formality score ({:.2}) below threshold ({:.2})", formality_score, self.formality_threshold),
                });
            }
        }

        // בדיקת מונחים טכניים
        if context.domain == Domain::Technical {
            let technical_terms_preserved = self.check_technical_terms(source, translation);
            if !technical_terms_preserved {
                results.push(QualityResult {
                    check_name: "technical_terms".to_string(),
                    passed: false,
                    message: "Technical terms not properly preserved".to_string(),
                });
            }
        }

        Ok(results)
    }

    fn calculate_formality_score(&self, _text: &str) -> f64 {
        // TODO: יישום חישוב ציון פורמליות
        0.9
    }

    fn check_technical_terms(&self, _source: &str, _translation: &str) -> bool {
        // TODO: יישום בדיקת שימור מונחים טכניים
        true
    }
} 