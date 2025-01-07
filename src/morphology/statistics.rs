use super::*;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use rayon::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordStats {
    pub frequency: f32,
    pub domains: HashMap<String, f32>,
    pub patterns: HashMap<String, f32>,
    pub contexts: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    pub frequency: f32,
    pub word_count: usize,
    pub domain_distribution: HashMap<String, f32>,
    pub common_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    pub word_count: usize,
    pub pattern_distribution: HashMap<String, f32>,
    pub register_distribution: HashMap<String, f32>,
}

#[derive(Debug)]
pub struct StatisticsAnalyzer {
    word_stats: HashMap<String, WordStats>,
    pattern_stats: HashMap<String, PatternStats>,
    domain_stats: HashMap<String, DomainStats>,
    total_words: usize,
}

impl StatisticsAnalyzer {
    pub fn new() -> Self {
        Self {
            word_stats: HashMap::new(),
            pattern_stats: HashMap::new(),
            domain_stats: HashMap::new(),
            total_words: 0,
        }
    }

    pub fn record_analysis(&mut self, word: &str, analysis: &EnhancedHebrewMorphology) {
        self.total_words += 1;
        
        // עדכון סטטיסטיקות מילה
        let word_stats = self.word_stats.entry(word.to_string())
            .or_insert_with(|| WordStats {
                frequency: 0.0,
                domains: HashMap::new(),
                patterns: HashMap::new(),
                contexts: HashMap::new(),
            });
        
        word_stats.frequency += 1.0;
        
        // עדכון סטטיסטיקות דפוס
        if let Some(pattern) = &analysis.basic.pattern {
            let pattern_stats = self.pattern_stats.entry(pattern.pattern.clone())
                .or_insert_with(|| PatternStats {
                    frequency: 0.0,
                    word_count: 0,
                    domain_distribution: HashMap::new(),
                    common_words: Vec::new(),
                });
            
            pattern_stats.frequency += 1.0;
            pattern_stats.word_count += 1;
            
            if !pattern_stats.common_words.contains(&word.to_string()) {
                pattern_stats.common_words.push(word.to_string());
            }
        }
        
        // עדכון סטטיסטיקות תחום
        if let Some(semantic_info) = &analysis.semantic_info {
            for domain in &semantic_info.domain {
                let domain_stats = self.domain_stats.entry(domain.clone())
                    .or_insert_with(|| DomainStats {
                        word_count: 0,
                        pattern_distribution: HashMap::new(),
                        register_distribution: HashMap::new(),
                    });
                
                domain_stats.word_count += 1;
                
                if let Some(pattern) = &analysis.basic.pattern {
                    *domain_stats.pattern_distribution
                        .entry(pattern.pattern.clone())
                        .or_insert(0.0) += 1.0;
                }
                
                *domain_stats.register_distribution
                    .entry(semantic_info.register.clone())
                    .or_insert(0.0) += 1.0;
            }
        }
    }

    pub fn normalize_statistics(&mut self) {
        // נרמול סטטיסטיקות מילים
        let total_words = self.total_words as f32;
        
        for stats in self.word_stats.values_mut() {
            stats.frequency /= total_words;
            
            // נרמול התפלגויות
            Self::normalize_distribution(&mut stats.domains);
            Self::normalize_distribution(&mut stats.patterns);
            Self::normalize_distribution(&mut stats.contexts);
        }
        
        // נרמול סטטיסטיקות דפוסים
        for stats in self.pattern_stats.values_mut() {
            stats.frequency /= total_words;
            Self::normalize_distribution(&mut stats.domain_distribution);
            
            // מיון מילים נפוצות לפי תדירות
            stats.common_words.sort_by(|a, b| {
                let freq_a = self.word_stats.get(a).map(|s| s.frequency).unwrap_or(0.0);
                let freq_b = self.word_stats.get(b).map(|s| s.frequency).unwrap_or(0.0);
                freq_b.partial_cmp(&freq_a).unwrap()
            });
            
            // שמירת רק 10 המילים הנפוצות ביותר
            stats.common_words.truncate(10);
        }
        
        // נרמול סטטיסטיקות תחומים
        for stats in self.domain_stats.values_mut() {
            Self::normalize_distribution(&mut stats.pattern_distribution);
            Self::normalize_distribution(&mut stats.register_distribution);
        }
    }

    fn normalize_distribution(dist: &mut HashMap<String, f32>) {
        let total: f32 = dist.values().sum();
        if total > 0.0 {
            for value in dist.values_mut() {
                *value /= total;
            }
        }
    }

    pub fn get_word_stats(&self, word: &str) -> Option<&WordStats> {
        self.word_stats.get(word)
    }

    pub fn get_pattern_stats(&self, pattern: &str) -> Option<&PatternStats> {
        self.pattern_stats.get(pattern)
    }

    pub fn get_domain_stats(&self, domain: &str) -> Option<&DomainStats> {
        self.domain_stats.get(domain)
    }

    pub fn get_most_common_patterns(&self, limit: usize) -> Vec<(String, f32)> {
        let mut patterns: Vec<_> = self.pattern_stats.iter()
            .map(|(pattern, stats)| (pattern.clone(), stats.frequency))
            .collect();
        
        patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        patterns.truncate(limit);
        patterns
    }

    pub fn get_domain_coverage(&self, domain: &str) -> f32 {
        if let Some(stats) = self.domain_stats.get(domain) {
            stats.word_count as f32 / self.total_words as f32
        } else {
            0.0
        }
    }

    pub fn get_pattern_complexity(&self, pattern: &str) -> Option<f32> {
        self.pattern_stats.get(pattern).map(|stats| {
            let domain_variety = stats.domain_distribution.len() as f32;
            let word_frequency = stats.frequency;
            // מדד מורכבות המשקל על פי מגוון התחומים ותדירות השימוש
            domain_variety * (1.0 - word_frequency)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_analysis() -> EnhancedHebrewMorphology {
        EnhancedHebrewMorphology {
            basic: HebrewMorphology {
                root: vec!['כ', 'ת', 'ב'],
                pattern: Some(HebrewPattern {
                    pattern: "קטל".to_string(),
                    description: "משקל פעל".to_string(),
                }),
                binyan: None,
                gender: None,
                number: None,
            },
            root_analysis: None,
            tense: None,
            person: None,
            is_construct_state: false,
            semantic_info: Some(SemanticInfo {
                domain: vec!["טכני".to_string()],
                register: "טכני_בינוני".to_string(),
                usage_examples: vec![],
            }),
            confidence_score: 0.8,
        }
    }

    #[test]
    fn test_record_analysis() {
        let mut analyzer = StatisticsAnalyzer::new();
        let analysis = create_test_analysis();
        
        analyzer.record_analysis("כתב", &analysis);
        assert_eq!(analyzer.total_words, 1);
        
        let word_stats = analyzer.get_word_stats("כתב");
        assert!(word_stats.is_some());
    }

    #[test]
    fn test_normalize_statistics() {
        let mut analyzer = StatisticsAnalyzer::new();
        let analysis = create_test_analysis();
        
        analyzer.record_analysis("כתב", &analysis);
        analyzer.normalize_statistics();
        
        let word_stats = analyzer.get_word_stats("כתב").unwrap();
        assert_eq!(word_stats.frequency, 1.0);
    }

    #[test]
    fn test_pattern_complexity() {
        let mut analyzer = StatisticsAnalyzer::new();
        let analysis = create_test_analysis();
        
        analyzer.record_analysis("כתב", &analysis);
        analyzer.normalize_statistics();
        
        let complexity = analyzer.get_pattern_complexity("קטל");
        assert!(complexity.is_some());
    }
} 