use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::morphology::{
    HebrewMorphology, RussianMorphology,
    HebrewAnalyzer, RussianAnalyzer,
    Gender, Number, Case,
};

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub text: String,
    pub grammar_score: f64,
    pub style_score: f64,
    pub terminology_score: f64,
    pub cultural_score: f64,
    pub neural_score: f64,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub issue_type: IssueType,
    pub description: String,
    pub severity: IssueSeverity,
    pub position: Option<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    Grammar,
    Style,
    Terminology,
    Cultural,
    Neural,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct QualityControl {
    hebrew_analyzer: Arc<HebrewAnalyzer>,
    russian_analyzer: Arc<RussianAnalyzer>,
    metrics: Arc<Mutex<QualityMetrics>>,
}

#[derive(Debug, Default)]
struct QualityMetrics {
    total_validations: usize,
    grammar_issues: usize,
    style_issues: usize,
    terminology_issues: usize,
    cultural_issues: usize,
    neural_issues: usize,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            grammar_score: 0.0,
            style_score: 0.0,
            terminology_score: 0.0,
            cultural_score: 0.0,
            neural_score: 0.0,
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|i| i.severity == IssueSeverity::Critical)
    }

    pub fn average_score(&self) -> f64 {
        (self.grammar_score + self.style_score + self.terminology_score +
         self.cultural_score + self.neural_score) / 5.0
    }
}

impl QualityControl {
    pub fn new() -> Self {
        Self {
            hebrew_analyzer: Arc::new(HebrewAnalyzer::new()),
            russian_analyzer: Arc::new(RussianAnalyzer::new()),
            metrics: Arc::new(Mutex::new(QualityMetrics::default())),
        }
    }

    pub async fn validate_deep(&self, text: &str) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        report.text = text.to_string();
        
        // בדיקות דקדוק
        self.validate_grammar(text, &mut report).await?;
        
        // בדיקות סגנון
        self.validate_style(text, &mut report).await?;
        
        // בדיקות מונחים
        self.validate_terminology(text, &mut report).await?;
        
        // בדיקות תרבותיות
        self.validate_cultural(text, &mut report).await?;
        
        // בדיקות נוירונים
        self.validate_neural(text, &mut report).await?;
        
        // עדכון מטריקות
        let mut metrics = self.metrics.lock().await;
        metrics.total_validations += 1;
        metrics.grammar_issues += report.issues.iter()
            .filter(|i| i.issue_type == IssueType::Grammar)
            .count();
        
        Ok(report)
    }

    async fn validate_grammar(&self, text: &str, report: &mut ValidationReport) -> Result<()> {
        // ניתוח מורפולוגי
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            // בדיקת התאמה דקדוקית
            if let Ok(hebrew) = self.hebrew_analyzer.analyze(word) {
                self.validate_hebrew_grammar(&hebrew, word, i, report)?;
            } else if let Ok(russian) = self.russian_analyzer.analyze(word) {
                self.validate_russian_grammar(&russian, word, i, report)?;
            }
        }
        
        // חישוב ציון דקדוק
        report.grammar_score = self.calculate_grammar_score(report);
        
        Ok(())
    }

    fn validate_hebrew_grammar(
        &self,
        analysis: &HebrewMorphology,
        word: &str,
        position: usize,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // בדיקת התאמת מין
        if let Some(gender) = &analysis.gender {
            if !self.validate_hebrew_gender_agreement(word, gender) {
                report.add_issue(ValidationIssue {
                    issue_type: IssueType::Grammar,
                    description: format!("חוסר התאמה במין דקדוקי: {}", word),
                    severity: IssueSeverity::Medium,
                    position: Some((position, position + word.len())),
                });
            }
        }
        
        // בדיקת התאמת מספר
        if let Some(number) = &analysis.number {
            if !self.validate_hebrew_number_agreement(word, number) {
                report.add_issue(ValidationIssue {
                    issue_type: IssueType::Grammar,
                    description: format!("חוסר התאמה במספר דקדוקי: {}", word),
                    severity: IssueSeverity::Medium,
                    position: Some((position, position + word.len())),
                });
            }
        }
        
        Ok(())
    }

    fn validate_russian_grammar(
        &self,
        analysis: &RussianMorphology,
        word: &str,
        position: usize,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // בדיקת התאמת מין
        if let Some(gender) = &analysis.gender {
            if !self.validate_russian_gender_agreement(word, gender) {
                report.add_issue(ValidationIssue {
                    issue_type: IssueType::Grammar,
                    description: format!("חוסר התאמה במין דקדוקי: {}", word),
                    severity: IssueSeverity::Medium,
                    position: Some((position, position + word.len())),
                });
            }
        }
        
        // בדיקת התאמת מספר
        if let Some(number) = &analysis.number {
            if !self.validate_russian_number_agreement(word, number) {
                report.add_issue(ValidationIssue {
                    issue_type: IssueType::Grammar,
                    description: format!("חוסר התאמה במספר דקדוקי: {}", word),
                    severity: IssueSeverity::Medium,
                    position: Some((position, position + word.len())),
                });
            }
        }
        
        // בדיקת התאמת מקרה דקדוקי
        if let Some(case) = &analysis.case {
            if !self.validate_russian_case_agreement(word, case) {
                report.add_issue(ValidationIssue {
                    issue_type: IssueType::Grammar,
                    description: format!("חוסר התאמה במקרה דקדוקי: {}", word),
                    severity: IssueSeverity::Medium,
                    position: Some((position, position + word.len())),
                });
            }
        }
        
        Ok(())
    }

    fn validate_hebrew_gender_agreement(&self, _word: &str, _gender: &Gender) -> bool {
        // TODO: יישום בדיקת התאמת מין בעברית
        true
    }

    fn validate_hebrew_number_agreement(&self, _word: &str, _number: &Number) -> bool {
        // TODO: יישום בדיקת התאמת מספר בעברית
        true
    }

    fn validate_russian_gender_agreement(&self, _word: &str, _gender: &Gender) -> bool {
        // TODO: יישום בדיקת התאמת מין ברוסית
        true
    }

    fn validate_russian_number_agreement(&self, _word: &str, _number: &Number) -> bool {
        // TODO: יישום בדיקת התאמת מספר ברוסית
        true
    }

    fn validate_russian_case_agreement(&self, _word: &str, _case: &Case) -> bool {
        // TODO: יישום בדיקת התאמת מקרה דקדוקי ברוסית
        true
    }

    fn calculate_grammar_score(&self, report: &ValidationReport) -> f64 {
        let grammar_issues = report.issues.iter()
            .filter(|i| i.issue_type == IssueType::Grammar)
            .count();
        
        if grammar_issues == 0 {
            1.0
        } else {
            let base_score = 1.0 - (grammar_issues as f64 * 0.1);
            base_score.max(0.0)
        }
    }

    async fn validate_style(&self, _text: &str, report: &mut ValidationReport) -> Result<()> {
        // TODO: יישום בדיקות סגנון
        report.style_score = 1.0;
        Ok(())
    }

    async fn validate_terminology(&self, _text: &str, report: &mut ValidationReport) -> Result<()> {
        // TODO: יישום בדיקות מונחים
        report.terminology_score = 1.0;
        Ok(())
    }

    async fn validate_cultural(&self, _text: &str, report: &mut ValidationReport) -> Result<()> {
        // TODO: יישום בדיקות תרבותיות
        report.cultural_score = 1.0;
        Ok(())
    }

    async fn validate_neural(&self, _text: &str, report: &mut ValidationReport) -> Result<()> {
        // TODO: יישום בדיקות נוירונים
        report.neural_score = 1.0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation() {
        let qc = QualityControl::new();
        let result = qc.validate_deep("ספר גדול").await;
        assert!(result.is_ok());
        
        let report = result.unwrap();
        assert!(!report.has_critical_issues());
        assert!(report.average_score() > 0.0);
    }

    #[tokio::test]
    async fn test_grammar_validation() {
        let qc = QualityControl::new();
        let mut report = ValidationReport::new();
        
        qc.validate_grammar("ספר גדול", &mut report).await.unwrap();
        assert!(report.grammar_score > 0.0);
        
        let grammar_issues = report.issues.iter()
            .filter(|i| i.issue_type == IssueType::Grammar)
            .count();
        assert_eq!(grammar_issues, 0);
    }

    #[tokio::test]
    async fn test_metrics() {
        let qc = QualityControl::new();
        let text = "ספר גדול";
        
        // בדיקה ראשונה
        qc.validate_deep(text).await.unwrap();
        
        let metrics = qc.metrics.lock().await;
        assert_eq!(metrics.total_validations, 1);
    }
} 