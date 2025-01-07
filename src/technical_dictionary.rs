use std::sync::Arc;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalTerm {
    pub source: String,
    pub target: String,
    pub domain: String,
    pub context: Vec<String>,
    pub usage_examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub metadata: TermMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermMetadata {
    pub confidence_score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
    pub verified: bool,
    pub source_references: Vec<String>,
}

pub struct TechnicalDictionary {
    terms: Arc<DashMap<String, TechnicalTerm>>,
    index: Arc<TermIndex>,
    learning_system: Arc<AdaptiveLearning>,
    validation_system: Arc<TermValidation>,
    context_analyzer: Arc<ContextAnalyzer>,
}

impl TechnicalDictionary {
    pub fn new() -> Self {
        Self {
            terms: Arc::new(DashMap::new()),
            index: Arc::new(TermIndex::new()),
            learning_system: Arc::new(AdaptiveLearning::new()),
            validation_system: Arc::new(TermValidation::new()),
            context_analyzer: Arc::new(ContextAnalyzer::new()),
        }
    }

    pub async fn add_term(&self, term: TechnicalTerm) -> Result<()> {
        // וידוא תקינות
        self.validation_system.validate_term(&term)?;
        
        // הוספה למילון
        self.terms.insert(term.source.clone(), term.clone());
        
        // עדכון אינדקס
        self.index.add_term(&term)?;
        
        // עדכון מערכת הלמידה
        self.learning_system.process_new_term(&term).await?;
        
        Ok(())
    }

    pub async fn find_term(&self, query: &str, context: Option<&str>) -> Result<Vec<TechnicalTerm>> {
        let mut results = Vec::new();
        
        // חיפוש מדויק
        if let Some(term) = self.terms.get(query) {
            results.push(term.clone());
        }
        
        // חיפוש דומים
        let similar_terms = self.index.find_similar(query)?;
        results.extend(similar_terms);
        
        // סינון לפי הקשר
        if let Some(context) = context {
            results = self.filter_by_context(&results, context)?;
        }
        
        // מיון לפי רלוונטיות
        self.sort_by_relevance(&mut results, query)?;
        
        Ok(results)
    }

    pub async fn update_term(&self, source: &str, updates: TermUpdates) -> Result<()> {
        if let Some(mut term) = self.terms.get_mut(source) {
            // עדכון השדות
            term.value_mut().apply_updates(updates)?;
            
            // וידוא לאחר עדכון
            self.validation_system.validate_term(term.value())?;
            
            // עדכון אינדקס
            self.index.update_term(term.value())?;
            
            // עדכון מערכת הלמידה
            self.learning_system.process_term_update(term.value()).await?;
        }
        
        Ok(())
    }

    pub async fn learn_from_usage(&self, text: &str, translation: &str) -> Result<()> {
        // זיהוי מונחים בטקסט
        let terms = self.identify_terms_in_text(text)?;
        
        // ניתוח הקשר
        let context = self.context_analyzer.analyze(text)?;
        
        // עדכון סטטיסטיקות שימוש
        for term in terms {
            self.update_usage_statistics(&term, &context).await?;
        }
        
        // למידה מהתרגום
        self.learning_system.learn_from_translation(text, translation, &terms).await?;
        
        Ok(())
    }

    fn filter_by_context(&self, terms: &[TechnicalTerm], context: &str) -> Result<Vec<TechnicalTerm>> {
        let context_vector = self.context_analyzer.vectorize(context)?;
        
        let filtered: Vec<_> = terms
            .iter()
            .filter(|term| {
                let term_context = term.context.join(" ");
                let term_vector = self.context_analyzer.vectorize(&term_context).unwrap_or_default();
                self.context_analyzer.calculate_similarity(&context_vector, &term_vector) > 0.7
            })
            .cloned()
            .collect();
        
        Ok(filtered)
    }

    fn sort_by_relevance(&self, terms: &mut [TechnicalTerm], query: &str) -> Result<()> {
        terms.sort_by(|a, b| {
            let score_a = self.calculate_relevance_score(a, query);
            let score_b = self.calculate_relevance_score(b, query);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(())
    }

    fn calculate_relevance_score(&self, term: &TechnicalTerm, query: &str) -> f64 {
        let mut score = 0.0;
        
        // התאמה טקסטואלית
        score += self.calculate_text_similarity(&term.source, query);
        
        // ציון אמון
        score += term.metadata.confidence_score;
        
        // תדירות שימוש
        score += (term.metadata.usage_count as f64).log10() / 10.0;
        
        // אימות
        if term.metadata.verified {
            score += 0.3;
        }
        
        score
    }
}

pub struct TermIndex {
    embeddings: HashMap<String, Vec<f32>>,
    index: Arc<faiss::Index>,
}

impl TermIndex {
    pub fn new() -> Self {
        Self {
            embeddings: HashMap::new(),
            index: Arc::new(faiss::Index::new(384, "Flat")),
        }
    }

    pub fn add_term(&self, term: &TechnicalTerm) -> Result<()> {
        let embedding = self.compute_embedding(&term.source)?;
        self.embeddings.insert(term.source.clone(), embedding.clone());
        self.index.add(&embedding)?;
        Ok(())
    }

    pub fn find_similar(&self, query: &str) -> Result<Vec<TechnicalTerm>> {
        let query_embedding = self.compute_embedding(query)?;
        let (distances, indices) = self.index.search(&query_embedding, 10)?;
        
        let mut results = Vec::new();
        for (distance, index) in distances.iter().zip(indices.iter()) {
            if let Some(term) = self.get_term_by_index(*index) {
                results.push(term.clone());
            }
        }
        
        Ok(results)
    }
} 