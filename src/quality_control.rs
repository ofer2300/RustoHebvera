use tch::{nn, Device, Tensor};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

pub struct QualityMetrics {
    accuracy: f64,
    fluency: f64,
    technical_accuracy: f64,
    cultural_appropriateness: f64,
    coherence: f64,
    style_consistency: f64,
    terminology_accuracy: f64,
    grammar_correctness: f64,
}

pub struct QualityControl {
    metrics: Arc<Mutex<QualityMetrics>>,
    rules_engine: Arc<RulesEngine>,
    neural_checker: Arc<NeuralQualityChecker>,
    style_analyzer: Arc<StyleAnalyzer>,
    grammar_checker: Arc<GrammarChecker>,
    terminology_validator: Arc<TerminologyValidator>,
    cultural_analyzer: Arc<CulturalAnalyzer>,
}

impl QualityControl {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(QualityMetrics {
                accuracy: 0.0,
                fluency: 0.0,
                technical_accuracy: 0.0,
                cultural_appropriateness: 0.0,
                coherence: 0.0,
                style_consistency: 0.0,
                terminology_accuracy: 0.0,
                grammar_correctness: 0.0,
            })),
            rules_engine: Arc::new(RulesEngine::new()),
            neural_checker: Arc::new(NeuralQualityChecker::new()),
            style_analyzer: Arc::new(StyleAnalyzer::new()),
            grammar_checker: Arc::new(GrammarChecker::new()),
            terminology_validator: Arc::new(TerminologyValidator::new()),
            cultural_analyzer: Arc::new(CulturalAnalyzer::new()),
        }
    }

    pub async fn validate_deep(&self, translation: &str) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // בדיקות מקבילות
        let (
            grammar_score,
            style_score,
            terminology_score,
            cultural_score,
            neural_score,
        ) = tokio::join!(
            self.grammar_checker.check(translation),
            self.style_analyzer.analyze(translation),
            self.terminology_validator.validate(translation),
            self.cultural_analyzer.analyze(translation),
            self.neural_checker.evaluate(translation),
        );

        // עדכון המדדים
        let mut metrics = self.metrics.lock().await;
        metrics.grammar_correctness = grammar_score?;
        metrics.style_consistency = style_score?;
        metrics.terminology_accuracy = terminology_score?;
        metrics.cultural_appropriateness = cultural_score?;
        
        // בדיקת קוהרנטיות
        let coherence_score = self.check_coherence(translation).await?;
        metrics.coherence = coherence_score;
        
        // חישוב ציון סופי
        let final_score = self.calculate_final_score(&metrics);
        
        // יצירת דוח מפורט
        report.add_score("דקדוק", metrics.grammar_correctness);
        report.add_score("סגנון", metrics.style_consistency);
        report.add_score("מונחים", metrics.terminology_accuracy);
        report.add_score("התאמה תרבותית", metrics.cultural_appropriateness);
        report.add_score("קוהרנטיות", metrics.coherence);
        report.add_score("ציון סופי", final_score);
        
        Ok(report)
    }

    async fn check_coherence(&self, text: &str) -> Result<f64> {
        let sentences = self.split_to_sentences(text);
        let mut coherence_score = 0.0;
        
        for window in sentences.windows(2) {
            let score = self.neural_checker.check_coherence(&window[0], &window[1]).await?;
            coherence_score += score;
        }
        
        Ok(coherence_score / (sentences.len() - 1) as f64)
    }

    fn calculate_final_score(&self, metrics: &QualityMetrics) -> f64 {
        let weights = [
            (metrics.grammar_correctness, 0.25),
            (metrics.style_consistency, 0.15),
            (metrics.terminology_accuracy, 0.20),
            (metrics.cultural_appropriateness, 0.15),
            (metrics.coherence, 0.25),
        ];
        
        weights.iter().map(|(score, weight)| score * weight).sum()
    }
}

pub struct ValidationReport {
    scores: HashMap<String, f64>,
    suggestions: Vec<String>,
    warnings: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            suggestions: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_score(&mut self, category: &str, score: f64) {
        self.scores.insert(category.to_string(), score);
    }

    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
} 