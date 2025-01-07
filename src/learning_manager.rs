use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::morphology::{HebrewMorphology, RussianMorphology};
use crate::quality_control::ValidationReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: LearningEventType,
    pub source_text: String,
    pub target_text: String,
    pub validation_report: Option<ValidationReport>,
    pub user_feedback: Option<UserFeedback>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEventType {
    Translation,
    Correction,
    Feedback,
    ValidationFailure,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub rating: u8,
    pub comments: Option<String>,
    pub corrections: Option<Vec<Correction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub original_text: String,
    pub corrected_text: String,
    pub correction_type: CorrectionType,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorrectionType {
    Grammar,
    Style,
    Terminology,
    Cultural,
    Other,
}

pub struct AdvancedLearningManager {
    events: Arc<Mutex<Vec<LearningEvent>>>,
    hebrew_patterns: Arc<Mutex<HebrewPatternLearner>>,
    russian_patterns: Arc<Mutex<RussianPatternLearner>>,
    feedback_analyzer: Arc<Mutex<FeedbackAnalyzer>>,
    domain_adapter: Arc<Mutex<DomainAdapter>>,
    continuous_learner: Arc<Mutex<ContinuousLearner>>,
    performance_monitor: Arc<Mutex<PerformanceMonitor>>,
    optimization_engine: Arc<Mutex<OptimizationEngine>>,
}

#[derive(Debug, Default)]
struct HebrewPatternLearner {
    morphology_patterns: Vec<(HebrewMorphology, f64)>,
    common_mistakes: Vec<(String, String, f64)>,
    style_patterns: Vec<(String, f64)>,
}

#[derive(Debug, Default)]
struct RussianPatternLearner {
    morphology_patterns: Vec<(RussianMorphology, f64)>,
    case_patterns: Vec<(String, String, f64)>,
    style_patterns: Vec<(String, f64)>,
}

#[derive(Debug, Default)]
struct FeedbackAnalyzer {
    total_feedback: usize,
    positive_feedback: usize,
    negative_feedback: usize,
    common_issues: Vec<(String, usize)>,
}

impl AdvancedLearningManager {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            hebrew_patterns: Arc::new(Mutex::new(HebrewPatternLearner::default())),
            russian_patterns: Arc::new(Mutex::new(RussianPatternLearner::default())),
            feedback_analyzer: Arc::new(Mutex::new(FeedbackAnalyzer::default())),
            domain_adapter: Arc::new(Mutex::new(DomainAdapter::new())),
            continuous_learner: Arc::new(Mutex::new(ContinuousLearner::new())),
            performance_monitor: Arc::new(Mutex::new(PerformanceMonitor::new())),
            optimization_engine: Arc::new(Mutex::new(OptimizationEngine::new())),
        }
    }

    pub async fn record_event(&self, event: LearningEvent) -> Result<EventMetrics> {
        // מדידת ביצועים
        let _perf = self.performance_monitor.lock().await.start_operation("record_event");
        
        // תיעוד האירוע
        let mut events = self.events.lock().await;
        events.push(event.clone());
        
        // ניתוח האירוע
        let metrics = match event.event_type {
            LearningEventType::Translation => {
                self.analyze_translation(&event).await?
            }
            LearningEventType::Correction => {
                self.analyze_correction(&event).await?
            }
            LearningEventType::Feedback => {
                self.analyze_feedback(&event).await?
            }
            LearningEventType::ValidationFailure => {
                self.analyze_failure(&event).await?
            }
            LearningEventType::Success => {
                self.analyze_success(&event).await?
            }
        };

        // אופטימיזציה מתמשכת
        self.optimization_engine.lock().await.optimize(&metrics).await?;
        
        Ok(metrics)
    }

    async fn analyze_translation(&self, event: &LearningEvent) -> Result<EventMetrics> {
        let mut metrics = EventMetrics::new();
        
        if let Some(report) = &event.validation_report {
            // ניתוח דפוסים מורפולוגיים
            let mut hebrew_patterns = self.hebrew_patterns.lock().await;
            let hebrew_metrics = hebrew_patterns.analyze_morphology_enhanced(
                &event.source_text,
                &event.target_text,
                report
            ).await?;
            metrics.merge(hebrew_metrics);
            
            let mut russian_patterns = self.russian_patterns.lock().await;
            let russian_metrics = russian_patterns.analyze_morphology_enhanced(
                &event.source_text,
                &event.target_text,
                report
            ).await?;
            metrics.merge(russian_metrics);
            
            // ניתוח דפוסי סגנון
            let style_metrics = self.analyze_style_patterns(
                &event.source_text,
                &event.target_text,
                report
            ).await?;
            metrics.merge(style_metrics);
            
            // למידה מתמשכת
            let mut learner = self.continuous_learner.lock().await;
            learner.learn_from_translation(event, &metrics).await?;
        }
        
        Ok(metrics)
    }

    async fn analyze_correction(&self, event: &LearningEvent) -> Result<EventMetrics> {
        let mut metrics = EventMetrics::new();
        
        if let Some(feedback) = &event.user_feedback {
            if let Some(corrections) = &feedback.corrections {
                for correction in corrections {
                    match correction.correction_type {
                        CorrectionType::Grammar => {
                            let grammar_metrics = self.analyze_grammar_correction(
                                correction,
                                &event.source_text,
                                &event.target_text
                            ).await?;
                            metrics.merge(grammar_metrics);
                        }
                        CorrectionType::Style => {
                            let style_metrics = self.analyze_style_correction(
                                correction,
                                &event.source_text,
                                &event.target_text
                            ).await?;
                            metrics.merge(style_metrics);
                        }
                        CorrectionType::Terminology => {
                            let term_metrics = self.analyze_terminology_correction(
                                correction,
                                &event.source_text,
                                &event.target_text
                            ).await?;
                            metrics.merge(term_metrics);
                        }
                        CorrectionType::Cultural => {
                            let cultural_metrics = self.analyze_cultural_correction(
                                correction,
                                &event.source_text,
                                &event.target_text
                            ).await?;
                            metrics.merge(cultural_metrics);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(metrics)
    }

    async fn analyze_feedback(&self, event: &LearningEvent) -> Result<EventMetrics> {
        let mut metrics = EventMetrics::new();
        
        if let Some(feedback) = &event.user_feedback {
            let mut analyzer = self.feedback_analyzer.lock().await;
            
            // עיתוח מתקדם של משוב
            let feedback_metrics = analyzer.analyze_feedback_enhanced(
                feedback,
                &event.source_text,
                &event.target_text
            ).await?;
            metrics.merge(feedback_metrics);
            
            // התאמת דומיין
            let mut domain_adapter = self.domain_adapter.lock().await;
            let domain_metrics = domain_adapter.adapt_from_feedback(
                feedback,
                &event.source_text,
                &event.target_text
            ).await?;
            metrics.merge(domain_metrics);
            
            // למידה מתמשכת
            let mut learner = self.continuous_learner.lock().await;
            learner.learn_from_feedback(feedback, &metrics).await?;
        }
        
        Ok(metrics)
    }

    pub async fn get_learning_statistics(&self) -> Result<EnhancedLearningStatistics> {
        let events = self.events.lock().await;
        let analyzer = self.feedback_analyzer.lock().await;
        let monitor = self.performance_monitor.lock().await;
        let optimization = self.optimization_engine.lock().await;
        
        Ok(EnhancedLearningStatistics {
            total_events: events.len(),
            success_rate: self.calculate_success_rate(&events),
            average_confidence: self.calculate_average_confidence(&events),
            positive_feedback_rate: self.calculate_feedback_rate(&analyzer),
            performance_metrics: monitor.get_metrics().await?,
            optimization_metrics: optimization.get_metrics().await?,
            domain_coverage: self.calculate_domain_coverage().await?,
            learning_progress: self.calculate_learning_progress().await?,
        })
    }
}

impl HebrewPatternLearner {
    fn analyze_morphology(&mut self, _source: &str, _target: &str) -> Result<()> {
        // TODO: יישום ניתוח מורפולוגי
        Ok(())
    }

    fn analyze_style(&mut self, _text: &str) -> Result<()> {
        // TODO: יישום ניתוח סגנון
        Ok(())
    }

    fn update_common_mistakes(&mut self, original: &str, corrected: &str) {
        if let Some(idx) = self.common_mistakes.iter()
            .position(|(o, c, _)| o == original && c == corrected) {
            self.common_mistakes[idx].2 += 1.0;
        } else {
            self.common_mistakes.push((
                original.to_string(),
                corrected.to_string(),
                1.0,
            ));
        }
    }

    fn update_style_patterns(&mut self, text: &str) {
        if let Some(idx) = self.style_patterns.iter()
            .position(|(p, _)| p == text) {
            self.style_patterns[idx].1 += 1.0;
        } else {
            self.style_patterns.push((text.to_string(), 1.0));
        }
    }

    fn analyze_failures(&mut self, _report: &ValidationReport) {
        // TODO: יישום ניתוח כשלים
    }

    fn analyze_successes(&mut self, _report: &ValidationReport) {
        // TODO: יישום ניתוח הצלחות
    }
}

impl RussianPatternLearner {
    fn analyze_morphology(&mut self, _source: &str, _target: &str) -> Result<()> {
        // TODO: יישום ניתוח מורפולוגי
        Ok(())
    }

    fn analyze_style(&mut self, _text: &str) -> Result<()> {
        // TODO: יישום ניתוח סגנון
        Ok(())
    }

    fn update_common_mistakes(&mut self, original: &str, corrected: &str) {
        if let Some(idx) = self.case_patterns.iter()
            .position(|(o, c, _)| o == original && c == corrected) {
            self.case_patterns[idx].2 += 1.0;
        } else {
            self.case_patterns.push((
                original.to_string(),
                corrected.to_string(),
                1.0,
            ));
        }
    }

    fn update_style_patterns(&mut self, text: &str) {
        if let Some(idx) = self.style_patterns.iter()
            .position(|(p, _)| p == text) {
            self.style_patterns[idx].1 += 1.0;
        } else {
            self.style_patterns.push((text.to_string(), 1.0));
        }
    }

    fn analyze_failures(&mut self, _report: &ValidationReport) {
        // TODO: יישום ניתוח כשלים
    }

    fn analyze_successes(&mut self, _report: &ValidationReport) {
        // TODO: יישום ניתוח הצלחות
    }
}

impl FeedbackAnalyzer {
    fn analyze_comments(&mut self, comments: &str) {
        // ניתוח פשוט של מילות מפתח בהערות
        let keywords = ["grammar", "style", "terminology", "cultural"];
        
        for keyword in keywords.iter() {
            if comments.to_lowercase().contains(keyword) {
                if let Some(idx) = self.common_issues.iter()
                    .position(|(issue, _)| issue == keyword) {
                    self.common_issues[idx].1 += 1;
                } else {
                    self.common_issues.push((keyword.to_string(), 1));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventMetrics {
    pub grammar_score: f64,
    pub style_score: f64,
    pub terminology_score: f64,
    pub cultural_score: f64,
    pub performance_metrics: PerformanceMetrics,
    pub optimization_metrics: OptimizationMetrics,
}

impl EventMetrics {
    pub fn new() -> Self {
        Self {
            grammar_score: 0.0,
            style_score: 0.0,
            terminology_score: 0.0,
            cultural_score: 0.0,
            performance_metrics: PerformanceMetrics::default(),
            optimization_metrics: OptimizationMetrics::default(),
        }
    }

    pub fn merge(&mut self, other: EventMetrics) {
        self.grammar_score = (self.grammar_score + other.grammar_score) / 2.0;
        self.style_score = (self.style_score + other.style_score) / 2.0;
        self.terminology_score = (self.terminology_score + other.terminology_score) / 2.0;
        self.cultural_score = (self.cultural_score + other.cultural_score) / 2.0;
        self.performance_metrics.merge(&other.performance_metrics);
        self.optimization_metrics.merge(&other.optimization_metrics);
    }
}

#[derive(Debug, Clone)]
pub struct EnhancedLearningStatistics {
    pub total_events: usize,
    pub success_rate: f64,
    pub average_confidence: f64,
    pub positive_feedback_rate: f64,
    pub performance_metrics: PerformanceMetrics,
    pub optimization_metrics: OptimizationMetrics,
    pub domain_coverage: DomainCoverage,
    pub learning_progress: LearningProgress,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub average_translation_time_ms: u64,
    pub peak_memory_usage_mb: u64,
    pub cache_hit_rate: f64,
    pub throughput_per_second: f64,
}

#[derive(Debug, Clone, Default)]
pub struct OptimizationMetrics {
    pub current_learning_rate: f64,
    pub parameter_updates: u64,
    pub gradient_norm: f64,
    pub optimization_steps: u64,
}

#[derive(Debug, Clone)]
pub struct DomainCoverage {
    pub total_domains: usize,
    pub active_domains: usize,
    pub coverage_percentage: f64,
    pub domain_specific_accuracy: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct LearningProgress {
    pub initial_performance: f64,
    pub current_performance: f64,
    pub improvement_rate: f64,
    pub learning_curve: Vec<(DateTime<Utc>, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_event() {
        let manager = AdvancedLearningManager::new();
        let event = LearningEvent {
            timestamp: Utc::now(),
            event_type: LearningEventType::Translation,
            source_text: "Hello".to_string(),
            target_text: "שלום".to_string(),
            validation_report: None,
            user_feedback: None,
            confidence_score: 0.9,
        };
        
        manager.record_event(event).await.unwrap();
        
        let stats = manager.get_learning_statistics().await.unwrap();
        assert_eq!(stats.total_events, 1);
    }

    #[tokio::test]
    async fn test_feedback_analysis() {
        let manager = AdvancedLearningManager::new();
        let event = LearningEvent {
            timestamp: Utc::now(),
            event_type: LearningEventType::Feedback,
            source_text: "Hello".to_string(),
            target_text: "שלום".to_string(),
            validation_report: None,
            user_feedback: Some(UserFeedback {
                rating: 5,
                comments: Some("Great translation!".to_string()),
                corrections: None,
            }),
            confidence_score: 0.9,
        };
        
        manager.record_event(event).await.unwrap();
        
        let stats = manager.get_learning_statistics().await.unwrap();
        assert!(stats.positive_feedback_rate > 0.0);
    }
} 