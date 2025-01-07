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

pub struct LearningManager {
    events: Arc<Mutex<Vec<LearningEvent>>>,
    hebrew_patterns: Arc<Mutex<HebrewPatternLearner>>,
    russian_patterns: Arc<Mutex<RussianPatternLearner>>,
    feedback_analyzer: Arc<Mutex<FeedbackAnalyzer>>,
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

impl LearningManager {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            hebrew_patterns: Arc::new(Mutex::new(HebrewPatternLearner::default())),
            russian_patterns: Arc::new(Mutex::new(RussianPatternLearner::default())),
            feedback_analyzer: Arc::new(Mutex::new(FeedbackAnalyzer::default())),
        }
    }

    pub async fn record_event(&self, event: LearningEvent) -> Result<()> {
        // תיעוד האירוע
        let mut events = self.events.lock().await;
        events.push(event.clone());
        
        // ניתוח האירוע ועדכון הלומדים
        match event.event_type {
            LearningEventType::Translation => {
                self.analyze_translation(&event).await?;
            }
            LearningEventType::Correction => {
                self.analyze_correction(&event).await?;
            }
            LearningEventType::Feedback => {
                self.analyze_feedback(&event).await?;
            }
            LearningEventType::ValidationFailure => {
                self.analyze_failure(&event).await?;
            }
            LearningEventType::Success => {
                self.analyze_success(&event).await?;
            }
        }
        
        Ok(())
    }

    async fn analyze_translation(&self, event: &LearningEvent) -> Result<()> {
        // ניתוח דפוסים בתרגום
        if let Some(report) = &event.validation_report {
            // ניתוח דפוסים מורפולוגיים
            let mut hebrew_patterns = self.hebrew_patterns.lock().await;
            hebrew_patterns.analyze_morphology(&event.source_text, &event.target_text)?;
            
            let mut russian_patterns = self.russian_patterns.lock().await;
            russian_patterns.analyze_morphology(&event.source_text, &event.target_text)?;
            
            // ניתוח דפוסי סגנון
            hebrew_patterns.analyze_style(&event.source_text)?;
            russian_patterns.analyze_style(&event.target_text)?;
        }
        
        Ok(())
    }

    async fn analyze_correction(&self, event: &LearningEvent) -> Result<()> {
        if let Some(feedback) = &event.user_feedback {
            if let Some(corrections) = &feedback.corrections {
                for correction in corrections {
                    match correction.correction_type {
                        CorrectionType::Grammar => {
                            // עדכון דפוסי טעויות דקדוקיות
                            let mut hebrew_patterns = self.hebrew_patterns.lock().await;
                            hebrew_patterns.update_common_mistakes(
                                &correction.original_text,
                                &correction.corrected_text,
                            );
                            
                            let mut russian_patterns = self.russian_patterns.lock().await;
                            russian_patterns.update_common_mistakes(
                                &correction.original_text,
                                &correction.corrected_text,
                            );
                        }
                        CorrectionType::Style => {
                            // עדכון דפוסי סגנון
                            let mut hebrew_patterns = self.hebrew_patterns.lock().await;
                            hebrew_patterns.update_style_patterns(&correction.corrected_text);
                            
                            let mut russian_patterns = self.russian_patterns.lock().await;
                            russian_patterns.update_style_patterns(&correction.corrected_text);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn analyze_feedback(&self, event: &LearningEvent) -> Result<()> {
        if let Some(feedback) = &event.user_feedback {
            let mut analyzer = self.feedback_analyzer.lock().await;
            
            // עדכון סטטיסטיקות משוב
            analyzer.total_feedback += 1;
            if feedback.rating >= 4 {
                analyzer.positive_feedback += 1;
            } else if feedback.rating <= 2 {
                analyzer.negative_feedback += 1;
            }
            
            // ניתוח הערות
            if let Some(comments) = &feedback.comments {
                analyzer.analyze_comments(comments);
            }
        }
        
        Ok(())
    }

    async fn analyze_failure(&self, event: &LearningEvent) -> Result<()> {
        if let Some(report) = &event.validation_report {
            // ניתוח כשלים ועדכון דפוסים
            let mut hebrew_patterns = self.hebrew_patterns.lock().await;
            hebrew_patterns.analyze_failures(report);
            
            let mut russian_patterns = self.russian_patterns.lock().await;
            russian_patterns.analyze_failures(report);
        }
        
        Ok(())
    }

    async fn analyze_success(&self, event: &LearningEvent) -> Result<()> {
        if let Some(report) = &event.validation_report {
            // ניתוח הצלחות ועדכון דפוסים
            let mut hebrew_patterns = self.hebrew_patterns.lock().await;
            hebrew_patterns.analyze_successes(report);
            
            let mut russian_patterns = self.russian_patterns.lock().await;
            russian_patterns.analyze_successes(report);
        }
        
        Ok(())
    }

    pub async fn get_learning_statistics(&self) -> Result<LearningStatistics> {
        let events = self.events.lock().await;
        let analyzer = self.feedback_analyzer.lock().await;
        
        Ok(LearningStatistics {
            total_events: events.len(),
            success_rate: self.calculate_success_rate(&events),
            average_confidence: self.calculate_average_confidence(&events),
            positive_feedback_rate: self.calculate_feedback_rate(&analyzer),
            common_issues: analyzer.common_issues.clone(),
        })
    }

    fn calculate_success_rate(&self, events: &[LearningEvent]) -> f64 {
        let successes = events.iter()
            .filter(|e| matches!(e.event_type, LearningEventType::Success))
            .count();
        
        if events.is_empty() {
            0.0
        } else {
            successes as f64 / events.len() as f64
        }
    }

    fn calculate_average_confidence(&self, events: &[LearningEvent]) -> f64 {
        if events.is_empty() {
            0.0
        } else {
            events.iter()
                .map(|e| e.confidence_score)
                .sum::<f64>() / events.len() as f64
        }
    }

    fn calculate_feedback_rate(&self, analyzer: &FeedbackAnalyzer) -> f64 {
        if analyzer.total_feedback == 0 {
            0.0
        } else {
            analyzer.positive_feedback as f64 / analyzer.total_feedback as f64
        }
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
pub struct LearningStatistics {
    pub total_events: usize,
    pub success_rate: f64,
    pub average_confidence: f64,
    pub positive_feedback_rate: f64,
    pub common_issues: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_event() {
        let manager = LearningManager::new();
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
        let manager = LearningManager::new();
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