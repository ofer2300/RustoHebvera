use tch::{nn, Tensor, Kind};
use crate::translation_models::TranslationError;
use std::f64;
use std::sync::Arc;

/// מנגנון תשומת לב משופר עם תמיכה במספר ראשים
pub struct MultiHeadAttention {
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
}

impl MultiHeadAttention {
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
        }
    }
    
    pub fn forward(&self, query: &Tensor, key: &Tensor, value: &Tensor, mask: Option<&Tensor>) -> Result<Tensor, AttentionError> {
        let batch_size = query.size()[0];
        let seq_length = query.size()[1];
        
        // העברה לינארית
        let query = self.query_net.forward(query);
        let key = self.key_net.forward(key);
        let value = self.value_net.forward(value);
        
        // חלוקה לראשים
        let query = self.split_heads(&query);
        let key = self.split_heads(&key);
        let value = self.split_heads(&value);
        
        // חישוב ציוני תשומת לב
        let attention_scores = if self.sparse_attention {
            self.compute_sparse_attention(&query, &key)
        } else {
            query.matmul(&key.transpose(-2, -1)) * self.scale
        };
        
        // החלת מסכה והיררכיה
        let attention_probs = if self.hierarchical {
            self.apply_hierarchical_mask(attention_scores, mask)
        } else {
            self.apply_mask(attention_scores, mask)
        }?;
        
        // דרופאאוט
        let attention_probs = self.attention_dropout.forward(&attention_probs, true);
        
        // חישוב הקשר
        let context = attention_probs.matmul(&value);
        
        // איחוד ראשים
        let output = self.combine_heads(&context);
        
        // העברה לינארית סופית
        let output = self.output_net.forward(&output);
        
        Ok(output)
    }
    
    fn compute_sparse_attention(&self, query: &Tensor, key: &Tensor) -> Tensor {
        // מימוש תשומת לב דלילה
        let local_attn = self.compute_local_attention(query, key);
        let global_attn = self.compute_global_attention(query, key);
        
        local_attn + global_attn
    }
    
    fn compute_local_attention(&self, query: &Tensor, key: &Tensor) -> Tensor {
        // חישוב תשומת לב מקומית עם חלון
        let window_size = 128;
        let seq_length = query.size()[2];
        
        let mut local_attn = Tensor::zeros_like(query);
        
        for i in 0..seq_length {
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2).min(seq_length);
            
            let q = query.slice(2, i, i + 1, 1);
            let k = key.slice(2, start, end, 1);
            
            let score = q.matmul(&k.transpose(-2, -1)) * self.scale;
            local_attn.slice_assign(2, i, i + 1, 1, &score);
        }
        
        local_attn
    }
    
    fn compute_global_attention(&self, query: &Tensor, key: &Tensor) -> Tensor {
        // חישוב תשומת לב גלובלית לטוקנים חשובים
        let importance = self.compute_token_importance(query);
        let top_k = importance.topk(self.num_heads, -1, true, true);
        
        let global_tokens = key.gather(-2, &top_k.indices, false);
        query.matmul(&global_tokens.transpose(-2, -1)) * self.scale
    }
    
    fn apply_hierarchical_mask(&self, scores: Tensor, mask: Option<&Tensor>) -> Result<Tensor, AttentionError> {
        // מימוש מסכה היררכית
        let mut masked = scores;
        
        if let Some(mask) = mask {
            // מסכה ברמה הראשונה - בלוקים
            let block_mask = self.create_block_mask(mask);
            masked = masked * block_mask;
            
            // מסכה ברמה השנייה - תוך-בלוקית
            let local_mask = self.create_local_mask(mask);
            masked = masked * local_mask;
        }
        
        Ok(masked.softmax(-1, Kind::Float))
    }
    
    fn split_heads(&self, x: &Tensor) -> Tensor {
        let batch_size = x.size()[0];
        let seq_length = x.size()[1];
        
        x.view([batch_size, seq_length, self.num_heads, self.head_dim])
         .transpose(1, 2)
    }
    
    fn combine_heads(&self, x: &Tensor) -> Tensor {
        let batch_size = x.size()[0];
        let seq_length = x.size()[2];
        
        x.transpose(1, 2)
         .contiguous()
         .view([batch_size, seq_length, self.num_heads * self.head_dim])
    }
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