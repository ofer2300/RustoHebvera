use std::sync::Arc;
use tch::{nn, Tensor, Device, Kind};
use crate::translation_models::{TranslationError, TranslationContext};
use super::vocabulary::Vocabulary;
use super::NeuralTranslator;

/// מנהל הלמידה המתמשכת
pub struct ContinuousLearningManager {
    translator: Arc<NeuralTranslator>,
    optimizer: nn::Optimizer,
    var_store: Arc<nn::VarStore>,
    learning_rate: f64,
    batch_size: i64,
    max_grad_norm: f64,
    device: Device,
    steps: i64,
    warmup_steps: i64,
    best_loss: f64,
    patience: i64,
    patience_counter: i64,
}

impl ContinuousLearningManager {
    pub fn new(
        translator: Arc<NeuralTranslator>,
        var_store: Arc<nn::VarStore>,
        config: LearningConfig
    ) -> Self {
        let optimizer = nn::Adam::default().build(&var_store, config.learning_rate).unwrap();
        
        Self {
            translator,
            optimizer,
            var_store,
            learning_rate: config.learning_rate,
            batch_size: config.batch_size,
            max_grad_norm: config.max_grad_norm,
            device: var_store.device(),
            steps: 0,
            warmup_steps: config.warmup_steps,
            best_loss: f64::INFINITY,
            patience: config.patience,
            patience_counter: 0,
        }
    }

    /// עדכון המודל על סמך דוגמה חדשה
    pub fn update(&mut self, source: &str, target: &str, context: &TranslationContext) -> Result<TrainingMetrics, TranslationError> {
        self.var_store.zero_grad();
        
        // חישוב Loss ועדכון משקולות
        let loss = self.compute_loss(source, target)?;
        loss.backward();
        
        // חיתוך גרדיאנטים למניעת התפוצצות
        self.var_store.clip_grad_norm(self.max_grad_norm);
        
        // עדכון קצב למידה לפי Warmup
        self.update_learning_rate_with_warmup();
        
        // עדכון המודל
        self.optimizer.step();
        
        // עדכון מונה הצעדים
        self.steps += 1;
        
        let loss_value = loss.double_value(&[]);
        self.update_early_stopping(loss_value);
        
        Ok(TrainingMetrics {
            loss: loss_value,
            perplexity: f64::exp(loss_value),
        })
    }

    /// אימון על אצווה של דוגמאות
    pub fn train_batch(&mut self, batch: &TrainingBatch) -> Result<TrainingMetrics, TranslationError> {
        self.var_store.zero_grad();
        
        let mut total_loss = 0.0;
        let mut sample_count = 0;
        
        for (source, target, _context) in batch.samples.iter() {
            let loss = self.compute_loss(source, target)?;
            total_loss += loss.double_value(&[]);
            sample_count += 1;
            
            loss.backward();
        }
        
        let avg_loss = total_loss / sample_count as f64;
        
        // חיתוך גרדיאנטים למניעת התפוצצות
        self.var_store.clip_grad_norm(self.max_grad_norm);
        
        // עדכון קצב למידה לפי Warmup
        self.update_learning_rate_with_warmup();
        
        // עדכון המודל
        self.optimizer.step();
        
        // עדכון מונה הצעדים
        self.steps += 1;
        
        self.update_early_stopping(avg_loss);
        
        Ok(TrainingMetrics {
            loss: avg_loss,
            perplexity: f64::exp(avg_loss),
        })
    }

    /// שמירת מצב האימון
    pub fn save_checkpoint(&self, path: &str) -> Result<(), TranslationError> {
        self.var_store
            .save(path)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בשמירת נקודת ביקורת: {}", e)))
    }

    /// טעינת מצב אימון
    pub fn load_checkpoint(&self, path: &str) -> Result<(), TranslationError> {
        self.var_store
            .load(path)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בטעינת נקודת ביקורת: {}", e)))
    }

    /// עדכון קצב הלמידה
    pub fn update_learning_rate(&mut self, new_rate: f64) {
        self.learning_rate = new_rate;
        self.optimizer.set_lr(new_rate);
    }

    /// בדיקה האם להפסיק את האימון
    pub fn should_stop(&self) -> bool {
        self.patience_counter >= self.patience
    }

    /// קבלת מספר הצעדים הנוכחי
    pub fn get_steps(&self) -> i64 {
        self.steps
    }

    /// קבלת ה-Loss הטוב ביותר
    pub fn get_best_loss(&self) -> f64 {
        self.best_loss
    }

    fn compute_loss(&self, source: &str, target: &str) -> Result<Tensor, TranslationError> {
        // המרת המשפטים לטנסורים
        let source_tensor = self.translator.prepare_input(&[source.to_string()])?;
        let target_tensor = self.translator.prepare_input(&[target.to_string()])?;
        
        // קידוד המשפט המקורי
        let encoded = self.translator.encoder.forward(&source_tensor)?;
        
        // יצירת מסיכת תשומת לב
        let target_len = target_tensor.size()[1];
        let attention_mask = Tensor::ones(&[1, target_len, target_len], (Kind::Bool, self.device))
            .triu(1)
            .to_kind(Kind::Bool);
        
        // פענוח והחלת Cross-Entropy Loss
        let decoded = self.translator.decoder.forward(&encoded, &target_tensor)?;
        
        // חישוב Loss
        let vocab_size = decoded.size()[2];
        let loss = decoded.view([-1, vocab_size])
            .cross_entropy_loss(
                &target_tensor.view([-1]),
                Some(self.translator.target_vocab.get_pad_index()),
                tch::Reduction::Mean,
            );
        
        Ok(loss)
    }

    fn update_learning_rate_with_warmup(&mut self) {
        if self.steps < self.warmup_steps {
            // Linear warmup
            let warmup_factor = self.steps as f64 / self.warmup_steps as f64;
            self.update_learning_rate(self.learning_rate * warmup_factor);
        } else {
            // Inverse square root decay
            let decay_factor = f64::sqrt(self.warmup_steps as f64 / self.steps as f64);
            self.update_learning_rate(self.learning_rate * decay_factor);
        }
    }

    fn update_early_stopping(&mut self, loss: f64) {
        if loss < self.best_loss {
            self.best_loss = loss;
            self.patience_counter = 0;
        } else {
            self.patience_counter += 1;
        }
    }
}

pub struct LearningConfig {
    pub learning_rate: f64,
    pub batch_size: i64,
    pub max_grad_norm: f64,
    pub warmup_steps: i64,
    pub patience: i64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            batch_size: 32,
            max_grad_norm: 1.0,
            warmup_steps: 4000,
            patience: 10,
        }
    }
}

pub struct TrainingMetrics {
    pub loss: f64,
    pub perplexity: f64,
}

pub struct TrainingBatch {
    pub samples: Vec<(String, String, TranslationContext)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::translation_models::{Domain, Style, Formality};

    fn create_test_context() -> TranslationContext {
        TranslationContext {
            domain: Domain::Technical,
            style: Style::Formal,
            formality: Formality::High,
            metadata: HashMap::new(),
        }
    }

    fn create_test_translator() -> Arc<NeuralTranslator> {
        let source_vocab = Arc::new(Vocabulary::new());
        let target_vocab = Arc::new(Vocabulary::new());
        
        let config = super::super::TranslatorConfig {
            hidden_size: 256,
            embedding_dim: 128,
            num_layers: 2,
            num_heads: 8,
            dropout: 0.1,
            source_vocab_size: source_vocab.size() as i64,
            target_vocab_size: target_vocab.size() as i64,
        };

        Arc::new(NeuralTranslator::new(config, source_vocab, target_vocab).unwrap())
    }

    fn create_test_var_store() -> Arc<nn::VarStore> {
        Arc::new(nn::VarStore::new(Device::Cpu))
    }

    #[test]
    fn test_learning_manager_creation() {
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let config = LearningConfig::default();
        
        let manager = ContinuousLearningManager::new(translator, var_store, config);
        assert_eq!(manager.learning_rate, 0.001);
        assert_eq!(manager.batch_size, 32);
        assert_eq!(manager.steps, 0);
        assert_eq!(manager.warmup_steps, 4000);
    }

    #[test]
    fn test_learning_rate_update() {
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let config = LearningConfig::default();
        
        let mut manager = ContinuousLearningManager::new(translator, var_store, config);
        let new_rate = 0.0005;
        manager.update_learning_rate(new_rate);
        assert_eq!(manager.learning_rate, new_rate);
    }

    #[test]
    fn test_checkpoint_save_load() {
        use tempfile::NamedTempFile;
        
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let config = LearningConfig::default();
        let manager = ContinuousLearningManager::new(translator, var_store, config);
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        
        assert!(manager.save_checkpoint(path).is_ok());
        assert!(manager.load_checkpoint(path).is_ok());
    }

    #[test]
    fn test_single_update() {
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let config = LearningConfig::default();
        let mut manager = ContinuousLearningManager::new(translator, var_store, config);
        
        let context = create_test_context();
        let result = manager.update("שלום", "привет", &context);
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.loss >= 0.0);
        assert!(metrics.perplexity >= 1.0);
        assert_eq!(manager.steps, 1);
    }

    #[test]
    fn test_batch_training() {
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let config = LearningConfig::default();
        let mut manager = ContinuousLearningManager::new(translator, var_store, config);
        
        let context = create_test_context();
        let batch = TrainingBatch {
            samples: vec![
                ("שלום".to_string(), "привет".to_string(), context.clone()),
                ("עולם".to_string(), "мир".to_string(), context.clone()),
            ],
        };
        
        let result = manager.train_batch(&batch);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.loss >= 0.0);
        assert!(metrics.perplexity >= 1.0);
        assert_eq!(manager.steps, 1);
    }

    #[test]
    fn test_loss_computation() {
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let config = LearningConfig::default();
        let manager = ContinuousLearningManager::new(translator, var_store, config);
        
        let loss = manager.compute_loss("שלום", "привет");
        assert!(loss.is_ok());
        let loss_tensor = loss.unwrap();
        assert!(loss_tensor.double_value(&[]) >= 0.0);
    }

    #[test]
    fn test_early_stopping() {
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let mut config = LearningConfig::default();
        config.patience = 2;
        
        let mut manager = ContinuousLearningManager::new(translator, var_store, config);
        
        // עדכון ראשון - Loss טוב יותר
        manager.update_early_stopping(1.0);
        assert_eq!(manager.best_loss, 1.0);
        assert_eq!(manager.patience_counter, 0);
        assert!(!manager.should_stop());
        
        // עדכון שני - Loss גרוע יותר
        manager.update_early_stopping(1.5);
        assert_eq!(manager.best_loss, 1.0);
        assert_eq!(manager.patience_counter, 1);
        assert!(!manager.should_stop());
        
        // עדכון שלישי - Loss גרוע יותר
        manager.update_early_stopping(2.0);
        assert_eq!(manager.best_loss, 1.0);
        assert_eq!(manager.patience_counter, 2);
        assert!(manager.should_stop());
    }

    #[test]
    fn test_warmup() {
        let translator = create_test_translator();
        let var_store = create_test_var_store();
        let mut config = LearningConfig::default();
        config.warmup_steps = 10;
        config.learning_rate = 0.001;
        
        let mut manager = ContinuousLearningManager::new(translator, var_store, config);
        
        // בדיקת Warmup
        manager.steps = 5;
        manager.update_learning_rate_with_warmup();
        assert!((manager.learning_rate - 0.0005).abs() < 1e-6);
        
        // בדיקת Decay
        manager.steps = 20;
        manager.update_learning_rate_with_warmup();
        assert!(manager.learning_rate < 0.001);
    }
} 