use tch::{nn, Tensor, Kind};
use crate::translation_models::TranslationError;
use std::f64;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::LruCache;

/// מנגנון תשומת לב משופר עם תמיכה במספר ראשים
pub struct EnhancedMultiHeadAttention {
    num_heads: i64,
    head_dim: i64,
    dropout: f64,
    scale: f64,
    query_net: Arc<nn::Linear>,
    key_net: Arc<nn::Linear>,
    value_net: Arc<nn::Linear>,
    output_net: Arc<nn::Linear>,
    attention_dropout: Arc<nn::Dropout>,
    sparse_attention: bool,
    hierarchical: bool,
    domain_specific: bool,
    morphology_aware: bool,
    context_cache: Arc<Mutex<LruCache<String, AttentionContext>>>,
}

impl EnhancedMultiHeadAttention {
    pub fn new(config: &AttentionConfig) -> Self {
        let head_dim = config.hidden_size / config.num_heads;
        let scale = 1.0 / f64::sqrt(head_dim as f64);
        let vs = nn::VarStore::new(Device::cuda_if_available());
        
        Self {
            num_heads: config.num_heads,
            head_dim,
            dropout: config.dropout,
            scale,
            query_net: Arc::new(nn::linear(&vs.root(), config.hidden_size, config.hidden_size, Default::default())),
            key_net: Arc::new(nn::linear(&vs.root(), config.hidden_size, config.hidden_size, Default::default())),
            value_net: Arc::new(nn::linear(&vs.root(), config.hidden_size, config.hidden_size, Default::default())),
            output_net: Arc::new(nn::linear(&vs.root(), config.hidden_size, config.hidden_size, Default::default())),
            attention_dropout: Arc::new(nn::Dropout::new(config.attention_dropout)),
            sparse_attention: config.sparse_attention,
            hierarchical: config.hierarchical,
            domain_specific: config.domain_specific,
            morphology_aware: config.morphology_aware,
            context_cache: Arc::new(Mutex::new(LruCache::new(1000))),
        }
    }

    pub async fn forward_enhanced(
        &self,
        query: &Tensor,
        key: &Tensor,
        value: &Tensor,
        mask: Option<&Tensor>,
        context: &AttentionContext,
    ) -> Result<(Tensor, AttentionMetrics), AttentionError> {
        let batch_size = query.size()[0];
        let seq_length = query.size()[1];
        
        // התאמה להקשר ותחום
        let (query_adapted, key_adapted, value_adapted) = self.adapt_inputs(
            query,
            key,
            value,
            context
        ).await?;

        // העברה לינארית עם התאמה מורפולוגית
        let query = self.query_net.forward_with_morphology(&query_adapted, &context.morphology)?;
        let key = self.key_net.forward_with_morphology(&key_adapted, &context.morphology)?;
        let value = self.value_net.forward_with_morphology(&value_adapted, &context.morphology)?;

        // חלוקה לראשים עם התאמה להקשר
        let query = self.split_heads_with_context(&query, context);
        let key = self.split_heads_with_context(&key, context);
        let value = self.split_heads_with_context(&value, context);

        // חישוב ציוני תשומת לב מותאמים
        let attention_scores = if self.sparse_attention {
            self.compute_sparse_attention(&query, &key, context).await?
        } else {
            self.compute_dense_attention(&query, &key, context).await?
        };

        // החלת מסכות והיררכיה
        let attention_probs = if self.hierarchical {
            self.apply_hierarchical_attention(attention_scores, mask, context).await?
        } else {
            self.apply_standard_attention(attention_scores, mask, context).await?
        };

        // דרופאאוט מותאם הקשר
        let attention_probs = self.attention_dropout.forward_with_context(&attention_probs, context);

        // חישוב הקשר עם אופטימיזציה
        let context_tensor = self.compute_context_tensor(&attention_probs, &value, context).await?;

        // איחוד ראשים עם התאמה להקשר
        let output = self.combine_heads_with_context(&context_tensor, context);

        // העברה לינארית סופית עם אופטימיזציה
        let output = self.output_net.forward_with_optimization(&output, context)?;

        // חישוב מטריקות
        let metrics = self.calculate_attention_metrics(
            &attention_probs,
            &context_tensor,
            context
        ).await?;

        Ok((output, metrics))
    }

    async fn adapt_inputs(
        &self,
        query: &Tensor,
        key: &Tensor,
        value: &Tensor,
        context: &AttentionContext,
    ) -> Result<(Tensor, Tensor, Tensor), AttentionError> {
        // התאמה לתחום ספציפי
        let (query, key, value) = if self.domain_specific {
            self.apply_domain_adaptation(query, key, value, &context.domain).await?
        } else {
            (query.shallow_clone(), key.shallow_clone(), value.shallow_clone())
        };

        // התאמה מורפולוגית
        let (query, key, value) = if self.morphology_aware {
            self.apply_morphology_adaptation(
                &query,
                &key,
                &value,
                &context.morphology
            ).await?
        } else {
            (query, key, value)
        };

        Ok((query, key, value))
    }

    async fn compute_sparse_attention(
        &self,
        query: &Tensor,
        key: &Tensor,
        context: &AttentionContext,
    ) -> Result<Tensor, AttentionError> {
        // חישוב תשומת לב מקומית
        let local_attn = self.compute_local_attention(query, key, context).await?;
        
        // חישוב תשומת לב גלובלית
        let global_attn = self.compute_global_attention(query, key, context).await?;
        
        // שילוב דינמי על פי הקשר
        let combination_weights = self.calculate_attention_weights(context).await?;
        
        Ok(local_attn * combination_weights.local + global_attn * combination_weights.global)
    }

    async fn compute_context_tensor(
        &self,
        attention_probs: &Tensor,
        value: &Tensor,
        context: &AttentionContext,
    ) -> Result<Tensor, AttentionError> {
        // בדיקת קאש
        let cache_key = self.generate_cache_key(attention_probs, value, context);
        let mut cache = self.context_cache.lock().await;
        
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(cached.tensor.shallow_clone());
        }

        // חישוב טנזור הקשר
        let context_tensor = attention_probs.matmul(value);
        
        // אופטימיזציה על פי הקשר
        let optimized = self.optimize_context_tensor(&context_tensor, context).await?;
        
        // שמירה בקאש
        cache.put(cache_key, AttentionContext {
            tensor: optimized.shallow_clone(),
            timestamp: std::time::SystemTime::now(),
        });

        Ok(optimized)
    }

    async fn calculate_attention_metrics(
        &self,
        attention_probs: &Tensor,
        context_tensor: &Tensor,
        attention_context: &AttentionContext,
    ) -> Result<AttentionMetrics, AttentionError> {
        Ok(AttentionMetrics {
            attention_entropy: self.calculate_attention_entropy(attention_probs)?,
            context_coverage: self.calculate_context_coverage(context_tensor)?,
            domain_relevance: self.calculate_domain_relevance(attention_context)?,
            morphology_accuracy: self.calculate_morphology_accuracy(attention_context)?,
        })
    }
}

#[derive(Debug)]
pub struct AttentionMetrics {
    pub attention_entropy: f64,
    pub context_coverage: f64,
    pub domain_relevance: f64,
    pub morphology_accuracy: f64,
}

#[derive(Debug)]
pub struct AttentionContext {
    pub domain: DomainInfo,
    pub morphology: MorphologyInfo,
    pub tensor: Tensor,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug)]
pub struct DomainInfo {
    pub name: String,
    pub confidence: f64,
    pub keywords: Vec<String>,
}

#[derive(Debug)]
pub struct MorphologyInfo {
    pub language: String,
    pub features: Vec<MorphologyFeature>,
    pub confidence: f64,
}

#[derive(Debug)]
pub enum MorphologyFeature {
    Gender(Gender),
    Number(Number),
    Case(Case),
    Tense(Tense),
    Person(Person),
}

pub struct SelfAttention {
    attention: MultiHeadAttention,
    layer_norm: LayerNorm,
}

impl SelfAttention {
    pub fn new(config: &TransformerConfig) -> Self {
        Self {
            attention: MultiHeadAttention::new(
                config.num_attention_heads,
                config.hidden_size,
                config.attention_probs_dropout_prob,
            ),
            layer_norm: LayerNorm::new(config.hidden_size),
        }
    }
    
    pub fn forward(&self, hidden_states: &Tensor, attention_mask: Option<&Tensor>) -> Tensor {
        let residual = hidden_states;
        
        // נורמליזציה
        let normalized = self.layer_norm.forward(hidden_states);
        
        // חישוב תשומת לב
        let attention_output = self.attention.forward(&normalized, &normalized, &normalized, attention_mask);
        
        // חיבור שארית
        attention_output + residual
    }
}

pub struct AttentionConfig {
    pub hidden_size: i64,
    pub num_heads: i64,
    pub dropout: f64,
    pub attention_dropout: f64,
    pub sparse_attention: bool,
    pub hierarchical: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tch::Device;

    fn create_test_config() -> AttentionConfig {
        AttentionConfig {
            hidden_size: 512,
            num_heads: 8,
            dropout: 0.1,
            attention_dropout: 0.1,
            sparse_attention: false,
            hierarchical: false,
        }
    }

    #[test]
    fn test_attention_creation() {
        let device = Device::Cpu;
        let vs = nn::VarStore::new(device);
        let config = create_test_config();
        
        let attention = MultiHeadAttention::new(&vs.root(), &config);
        assert_eq!(attention.num_heads, config.num_heads);
        assert_eq!(attention.head_dim, config.hidden_size / config.num_heads);
    }

    #[test]
    fn test_attention_forward() {
        let device = Device::Cpu;
        let vs = nn::VarStore::new(device);
        let config = create_test_config();
        
        let attention = MultiHeadAttention::new(&vs.root(), &config);
        
        let batch_size = 2;
        let seq_len = 10;
        let hidden_size = config.hidden_size;
        
        let query = Tensor::rand(&[batch_size, seq_len, hidden_size], (Kind::Float, device));
        let key = Tensor::rand(&[batch_size, seq_len, hidden_size], (Kind::Float, device));
        let value = Tensor::rand(&[batch_size, seq_len, hidden_size], (Kind::Float, device));
        
        let result = attention.forward(&query, &key, &value, None);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.size(), &[batch_size, seq_len, hidden_size]);
    }

    #[test]
    fn test_attention_with_mask() {
        let device = Device::Cpu;
        let vs = nn::VarStore::new(device);
        let config = create_test_config();
        
        let attention = MultiHeadAttention::new(&vs.root(), &config);
        
        let batch_size = 2;
        let seq_len = 10;
        let hidden_size = config.hidden_size;
        
        let query = Tensor::rand(&[batch_size, seq_len, hidden_size], (Kind::Float, device));
        let key = Tensor::rand(&[batch_size, seq_len, hidden_size], (Kind::Float, device));
        let value = Tensor::rand(&[batch_size, seq_len, hidden_size], (Kind::Float, device));
        let mask = Tensor::zeros(&[batch_size, seq_len], (Kind::Bool, device));
        
        let result = attention.forward(&query, &key, &value, Some(&mask));
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.size(), &[batch_size, seq_len, hidden_size]);
    }

    #[test]
    fn test_attention_with_sparse() {
        let config = AttentionConfig {
            hidden_size: 512,
            num_heads: 8,
            dropout: 0.1,
            attention_dropout: 0.1,
            sparse_attention: true,
            hierarchical: false,
        };
        
        let attention = MultiHeadAttention::new(&config);
        
        let batch_size = 2;
        let seq_length = 512;
        let device = Device::Cpu;
        
        let query = Tensor::rand(&[batch_size, seq_length, config.hidden_size], (Kind::Float, device));
        let key = Tensor::rand(&[batch_size, seq_length, config.hidden_size], (Kind::Float, device));
        let value = Tensor::rand(&[batch_size, seq_length, config.hidden_size], (Kind::Float, device));
        
        let result = attention.forward(&query, &key, &value, None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_attention_with_hierarchical() {
        let config = AttentionConfig {
            hidden_size: 512,
            num_heads: 8,
            dropout: 0.1,
            attention_dropout: 0.1,
            sparse_attention: false,
            hierarchical: true,
        };
        
        let attention = MultiHeadAttention::new(&config);
        
        let batch_size = 2;
        let seq_length = 512;
        let device = Device::Cpu;
        
        let query = Tensor::rand(&[batch_size, seq_length, config.hidden_size], (Kind::Float, device));
        let key = Tensor::rand(&[batch_size, seq_length, config.hidden_size], (Kind::Float, device));
        let value = Tensor::rand(&[batch_size, seq_length, config.hidden_size], (Kind::Float, device));
        let mask = Tensor::ones(&[batch_size, seq_length], (Kind::Bool, device));
        
        let result = attention.forward(&query, &key, &value, Some(&mask));
        assert!(result.is_ok());
    }
} 