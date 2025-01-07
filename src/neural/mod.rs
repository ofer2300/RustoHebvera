use std::sync::Arc;
use tch::{nn, Device, Tensor, Kind};
use crate::translation_models::TranslationError;
use super::vocabulary::{Vocabulary, VocabularyError};
use super::attention::{MultiHeadAttention, AttentionConfig};

/// מודל נוירוני בסיסי לתרגום
pub struct NeuralTranslator {
    encoder: Encoder,
    decoder: Decoder,
    embedding: Embedding,
    device: Device,
    source_vocab: Arc<Vocabulary>,
    target_vocab: Arc<Vocabulary>,
}

/// שכבת הקידוד
struct Encoder {
    lstm: nn::LSTM,
    dropout: f64,
    embedding: Arc<Embedding>,
    self_attention: MultiHeadAttention,
}

/// שכבת הפענוח
struct Decoder {
    lstm: nn::LSTM,
    output_layer: nn::Linear,
    dropout: f64,
    embedding: Arc<Embedding>,
    self_attention: MultiHeadAttention,
    cross_attention: MultiHeadAttention,
}

/// שכבת ה-Embedding
struct Embedding {
    encoder_embedding: nn::Embedding,
    decoder_embedding: nn::Embedding,
    embedding_dim: i64,
}

impl NeuralTranslator {
    pub fn new(
        config: TranslatorConfig,
        source_vocab: Arc<Vocabulary>,
        target_vocab: Arc<Vocabulary>,
    ) -> Result<Self, TranslationError> {
        let device = Device::cuda_if_available();
        let vs = nn::VarStore::new(device);
        
        let attention_config = AttentionConfig {
            hidden_size: config.hidden_size,
            num_heads: config.num_heads,
            dropout: config.dropout,
        };

        let embedding = Arc::new(Embedding::new(&vs.root(), &config));
        let encoder = Encoder::new(&vs.root(), &config, &attention_config, embedding.clone());
        let decoder = Decoder::new(&vs.root(), &config, &attention_config, embedding.clone());

        Ok(Self {
            encoder,
            decoder,
            embedding: Arc::try_unwrap(embedding).unwrap_or_else(|arc| (*arc).clone()),
            device,
            source_vocab,
            target_vocab,
        })
    }

    pub fn translate(&self, input: &[String]) -> Result<Vec<String>, TranslationError> {
        // המרת הקלט לטנסורים
        let input_tensor = self.prepare_input(input)?;
        
        // קידוד
        let encoded = self.encoder.forward(&input_tensor)?;
        
        // חצירת טנסור התחלתי לפענוח
        let batch_size = input_tensor.size()[0];
        let mut decoder_input = Tensor::full(
            &[batch_size, 1],
            self.target_vocab.get_bos_index() as i64,
            (Kind::Int64, self.device)
        );
        
        let max_length = 100; // אורך מקסימלי לתרגום
        let mut outputs = Vec::new();
        
        // פענוח אוטורגרסיבי
        for _ in 0..max_length {
            // פענוח צעד אחד
            let step_output = self.decoder.forward(&encoded, &decoder_input)?;
            
            // בחירת המילה הבאה
            let next_words = step_output
                .i((.., -1, ..))
                .argmax(-1, false)
                .to_kind(Kind::Int64);
            
            // הוספה לפלט
            outputs.push(next_words.copy());
            
            // בדיקה אם הגענו לסוף המשפט
            let is_eos = next_words.eq(self.target_vocab.get_eos_index());
            if is_eos.all().totype(Kind::Bool).into() {
                break;
            }
            
            // עדכון הקלט לצעד הבא
            decoder_input = Tensor::cat(&[decoder_input, next_words.view([-1, 1])], 1);
        }
        
        // שילוב כל הצעדים לטנסור אחד
        let output_tensor = Tensor::stack(&outputs, 1);
        
        // המרה חזרה למילים
        self.tensor_to_text(&output_tensor)
    }

    fn prepare_input(&self, input: &[String]) -> Result<Tensor, TranslationError> {
        let batch_size = input.len() as i64;
        let max_length = input.iter().map(|s| s.len()).max().unwrap() as i64;
        
        // יצירת טנסור אפסים
        let mut tensor = Tensor::zeros(&[batch_size, max_length], (Kind::Int64, self.device));
        
        // המרת המילים למספרים
        for (i, sentence) in input.iter().enumerate() {
            for (j, word) in sentence.split_whitespace().enumerate() {
                let word_idx = match self.source_vocab.get_index(word) {
                    Ok(idx) => idx,
                    Err(VocabularyError::WordNotFound(_)) => self.source_vocab.get_unk_index(),
                    Err(e) => return Err(TranslationError::VocabularyError(e.to_string())),
                };
                tensor.i((i as i64, j as i64)).copy_(&Tensor::from(word_idx));
            }
        }
        
        Ok(tensor)
    }

    fn tensor_to_text(&self, tensor: &Tensor) -> Result<Vec<String>, TranslationError> {
        let batch_size = tensor.size()[0];
        let mut result = Vec::with_capacity(batch_size as usize);
        
        for i in 0..batch_size {
            let mut sentence = Vec::new();
            let sequence = tensor.i(i);
            
            for j in 0..sequence.size()[0] {
                let word_idx = sequence.i(j).int64_value(&[]) as i64;
                let word = match self.target_vocab.get_word(word_idx) {
                    Ok(w) => w,
                    Err(e) => return Err(TranslationError::VocabularyError(e.to_string())),
                };
                if word == "<EOS>" {
                    break;
                }
                if word != "<PAD>" && word != "<BOS>" {
                    sentence.push(word);
                }
            }
            
            result.push(sentence.join(" "));
        }
        
        Ok(result)
    }
}

/// תצורת המודל הנוירוני
pub struct TranslatorConfig {
    pub hidden_size: i64,
    pub embedding_dim: i64,
    pub num_layers: i64,
    pub num_heads: i64,
    pub dropout: f64,
    pub source_vocab_size: i64,
    pub target_vocab_size: i64,
}

impl Encoder {
    fn new(vs: &nn::Path, config: &TranslatorConfig, attention_config: &AttentionConfig, embedding: Arc<Embedding>) -> Self {
        let lstm = nn::lstm(vs, config.embedding_dim, config.hidden_size, Default::default());
        let self_attention = MultiHeadAttention::new(vs, attention_config);
        
        Self {
            lstm,
            dropout: config.dropout,
            embedding,
            self_attention,
        }
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor, TranslationError> {
        // העברת הקלט דרך שכבת ה-Embedding
        let embedded = self.embedding.encoder_embedding.forward(input);
        
        // החלת Dropout
        let dropped = embedded.dropout(self.dropout, true);
        
        // העברה דרך ה-LSTM
        let (output, _) = self.lstm.forward(&dropped);
        
        // החלת מנגנון תשומת הלב
        let attended = self.self_attention.forward(&output, &output, &output, None)?;
        
        Ok(attended)
    }
}

impl Decoder {
    fn new(vs: &nn::Path, config: &TranslatorConfig, attention_config: &AttentionConfig, embedding: Arc<Embedding>) -> Self {
        let lstm = nn::lstm(vs, config.embedding_dim, config.hidden_size, Default::default());
        let output_layer = nn::linear(vs, config.hidden_size, config.target_vocab_size, Default::default());
        let self_attention = MultiHeadAttention::new(vs, attention_config);
        let cross_attention = MultiHeadAttention::new(vs, attention_config);
        
        Self {
            lstm,
            output_layer,
            dropout: config.dropout,
            embedding,
            self_attention,
            cross_attention,
        }
    }

    fn forward(&self, encoded: &Tensor, decoder_input: &Tensor) -> Result<Tensor, TranslationError> {
        // העברת הקלט דרך שכבת ה-Embedding
        let embedded = self.embedding.decoder_embedding.forward(decoder_input);
        
        // החלת Dropout
        let dropped = embedded.dropout(self.dropout, true);
        
        // העברה דרך ה-LSTM
        let (output, _) = self.lstm.forward(&dropped);
        
        // תשומת לב עצמית
        let self_attended = self.self_attention.forward(&output, &output, &output, None)?;
        
        // תשומת לב צולבת עם הקידוד
        let cross_attended = self.cross_attention.forward(&self_attended, encoded, encoded, None)?;
        
        // העברה דרך שכבת הפלט
        let logits = self.output_layer.forward(&cross_attended);
        
        Ok(logits)
    }
}

impl Embedding {
    fn new(vs: &nn::Path, config: &TranslatorConfig) -> Self {
        let encoder_embedding = nn::embedding(vs, config.source_vocab_size, config.embedding_dim, Default::default());
        let decoder_embedding = nn::embedding(vs, config.target_vocab_size, config.embedding_dim, Default::default());
        
        Self {
            encoder_embedding,
            decoder_embedding,
            embedding_dim: config.embedding_dim,
        }
    }
}

impl Attention {
    fn new(vs: &nn::Path, config: &TranslatorConfig) -> Self {
        let attention_layer = nn::linear(vs, config.hidden_size * 2, config.hidden_size, Default::default());
        let combine_layer = nn::linear(vs, config.hidden_size * 2, config.hidden_size, Default::default());
        
        Self {
            attention_layer,
            combine_layer,
        }
    }

    fn calculate(&self, encoded: &Tensor) -> Result<Tensor, TranslationError> {
        // חישוב ציוני תשומת לב
        let scores = self.attention_layer.forward(encoded);
        
        // נרמול באמצעות softmax
        let weights = scores.softmax(-1, Kind::Float);
        
        // הכפלה בקידוד המקורי
        let context = weights.matmul(encoded);
        
        // שילוב עם הקידוד המקורי
        let combined = Tensor::cat(&[encoded, context], -1);
        let output = self.combine_layer.forward(&combined);
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_vocab() -> Arc<Vocabulary> {
        let mut vocab = Vocabulary::new();
        vocab.add_word("שלום");
        vocab.add_word("עולם");
        vocab.add_word("אני");
        vocab.add_word("אוהב");
        vocab.add_word("לתכנת");
        vocab.add_word("привет");
        vocab.add_word("мир");
        vocab.add_word("я");
        vocab.add_word("люблю");
        vocab.add_word("программировать");
        Arc::new(vocab)
    }

    fn create_test_config(vocab_size: i64) -> TranslatorConfig {
        TranslatorConfig {
            hidden_size: 256,
            embedding_dim: 128,
            num_layers: 2,
            num_heads: 8,
            dropout: 0.1,
            source_vocab_size: vocab_size,
            target_vocab_size: vocab_size,
        }
    }

    #[test]
    fn test_neural_translator_creation() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        );

        assert!(translator.is_ok());
    }

    #[test]
    fn test_translation() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let input = vec!["שלום עולם".to_string()];
        let result = translator.translate(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_word_handling() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let input = vec!["מילה_לא_קיימת".to_string()];
        let result = translator.translate(&input);
        assert!(result.is_ok()); // אמור להשתמש בטוקן UNK
    }

    #[test]
    fn test_batch_translation() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let input = vec![
            "שלום עולם".to_string(),
            "привет мир".to_string(),
        ];
        let result = translator.translate(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_long_sequence_translation() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let input = vec!["אני אוהב לתכנת בשפת ראסט".to_string()];
        let result = translator.translate(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_input() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let input = vec!["".to_string()];
        let result = translator.translate(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_special_tokens_handling() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let input = vec!["<PAD> שלום <UNK>".to_string()];
        let result = translator.translate(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encoder_output_shape() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let input = vec!["שלום עולם".to_string()];
        let input_tensor = translator.prepare_input(&input).unwrap();
        let encoded = translator.encoder.forward(&input_tensor).unwrap();
        
        assert_eq!(encoded.size()[0], 1); // batch size
        assert_eq!(encoded.size()[2] as i64, config.hidden_size); // hidden size
    }

    #[test]
    fn test_decoder_output_shape() {
        let source_vocab = create_test_vocab();
        let target_vocab = create_test_vocab();
        let config = create_test_config(source_vocab.size() as i64);

        let translator = NeuralTranslator::new(
            config,
            source_vocab.clone(),
            target_vocab.clone(),
        ).unwrap();

        let batch_size = 1;
        let seq_len = 5;
        let device = Device::Cpu;

        let encoded = Tensor::zeros(&[batch_size, seq_len, config.hidden_size], (Kind::Float, device));
        let decoder_input = Tensor::zeros(&[batch_size, seq_len], (Kind::Int64, device));
        
        let output = translator.decoder.forward(&encoded, &decoder_input).unwrap();
        
        assert_eq!(output.size()[0], batch_size);
        assert_eq!(output.size()[2] as i64, config.target_vocab_size);
    }
} 