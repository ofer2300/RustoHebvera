use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::translation_models::TranslationError;

/// מידע על מודל שמור
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// שם המודל
    pub name: String,
    /// תיאור המודל
    pub description: String,
    /// גרסת המודל
    pub version: String,
    /// תאריך יצירה
    pub created_at: u64,
    /// תאריך עדכון אחרון
    pub updated_at: u64,
    /// מטריקות ביצועים
    pub metrics: ModelMetrics,
    /// תצורת המודל
    pub config: ModelConfig,
}

/// מטריקות ביצועים של המודל
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// דיוק התרגום
    pub translation_accuracy: f64,
    /// ציון BLEU
    pub bleu_score: f64,
    /// Loss סופי
    pub final_loss: f64,
    /// מספר צעדי אימון
    pub training_steps: i64,
}

/// תצורת המודל
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    /// גודל אוצר המילים
    pub vocab_size: i64,
    /// מספר השכבות
    pub num_layers: i64,
    /// גודל המודל
    pub model_size: i64,
    /// גודל ה-Embedding
    pub embedding_dim: i64,
    /// מספר ראשי תשומת הלב
    pub num_heads: i64,
}

/// מנהל אחסון המודלים
pub struct ModelStorage {
    base_path: PathBuf,
}

impl ModelStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> io::Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    /// שמירת מודל חדש
    pub fn save_model(
        &self,
        name: &str,
        description: &str,
        version: &str,
        model_path: &Path,
        metrics: ModelMetrics,
        config: ModelConfig,
    ) -> Result<(), TranslationError> {
        let model_dir = self.base_path.join(name).join(version);
        fs::create_dir_all(&model_dir)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה ביצירת תיקיית מודל: {}", e)))?;

        // שמירת המטא-דאטה
        let metadata = ModelMetadata {
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics,
            config,
        };

        let metadata_path = model_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בהמרת מטא-דאטה: {}", e)))?;
        
        fs::write(&metadata_path, metadata_json)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בשמירת מטא-דאטה: {}", e)))?;

        // העתקת קבצי המודל
        let model_dest = model_dir.join("model.pt");
        fs::copy(model_path, &model_dest)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בהעתקת המודל: {}", e)))?;

        Ok(())
    }

    /// טעינת מודל
    pub fn load_model(&self, name: &str, version: &str) -> Result<(PathBuf, ModelMetadata), TranslationError> {
        let model_dir = self.base_path.join(name).join(version);
        let metadata_path = model_dir.join("metadata.json");
        let model_path = model_dir.join("model.pt");

        // טעינת המטא-דאטה
        let metadata_json = fs::read_to_string(&metadata_path)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בטעינת מטא-דאטה: {}", e)))?;
        
        let metadata: ModelMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בפענוח מטא-דאטה: {}", e)))?;

        // בדיקה שקובץ המודל קיים
        if !model_path.exists() {
            return Err(TranslationError::LearningError(
                format!("קובץ המודל לא נמצא: {}", model_path.display())
            ));
        }

        Ok((model_path, metadata))
    }

    /// קבלת רשימת המודלים השמורים
    pub fn list_models(&self) -> Result<Vec<ModelMetadata>, TranslationError> {
        let mut models = Vec::new();

        for model_entry in fs::read_dir(&self.base_path)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בקריאת תיקיית מודלים: {}", e)))? {
            let model_entry = model_entry
                .map_err(|e| TranslationError::LearningError(format!("שגיאה בקריאת רשומת מודל: {}", e)))?;
            
            if !model_entry.file_type()
                .map_err(|e| TranslationError::LearningError(format!("שגיאה בקריאת סוג קובץ: {}", e)))?
                .is_dir() {
                continue;
            }

            for version_entry in fs::read_dir(model_entry.path())
                .map_err(|e| TranslationError::LearningError(format!("שגיאה בקריאת תיקיית גרסאות: {}", e)))? {
                let version_entry = version_entry
                    .map_err(|e| TranslationError::LearningError(format!("שגיאה בקריאת רשומת גרסה: {}", e)))?;
                
                let metadata_path = version_entry.path().join("metadata.json");
                if metadata_path.exists() {
                    let metadata_json = fs::read_to_string(&metadata_path)
                        .map_err(|e| TranslationError::LearningError(format!("שגיאה בטעינת מטא-דאטה: {}", e)))?;
                    
                    let metadata: ModelMetadata = serde_json::from_str(&metadata_json)
                        .map_err(|e| TranslationError::LearningError(format!("שגיאה בפענוח מטא-דאטה: {}", e)))?;
                    
                    models.push(metadata);
                }
            }
        }

        Ok(models)
    }

    /// מחיקת מודל
    pub fn delete_model(&self, name: &str, version: &str) -> Result<(), TranslationError> {
        let model_dir = self.base_path.join(name).join(version);
        if model_dir.exists() {
            fs::remove_dir_all(&model_dir)
                .map_err(|e| TranslationError::LearningError(format!("שגיאה במחיקת מודל: {}", e)))?;
            
            // מחיקת תיקיית המודל אם היא ריקה
            let parent_dir = model_dir.parent().unwrap();
            if fs::read_dir(parent_dir)
                .map_err(|e| TranslationError::LearningError(format!("שגיאה בקריאת תיקייה: {}", e)))?
                .count() == 0 {
                fs::remove_dir(parent_dir)
                    .map_err(|e| TranslationError::LearningError(format!("שגיאה במחיקת תיקייה: {}", e)))?;
            }
        }
        Ok(())
    }

    /// עדכון מטריקות של מודל
    pub fn update_metrics(&self, name: &str, version: &str, metrics: ModelMetrics) -> Result<(), TranslationError> {
        let model_dir = self.base_path.join(name).join(version);
        let metadata_path = model_dir.join("metadata.json");

        // טעינת המטא-דאטה הקיים
        let mut metadata: ModelMetadata = {
            let metadata_json = fs::read_to_string(&metadata_path)
                .map_err(|e| TranslationError::LearningError(format!("שגיאה בטעינת מטא-דאטה: {}", e)))?;
            
            serde_json::from_str(&metadata_json)
                .map_err(|e| TranslationError::LearningError(format!("שגיאה בפענוח מטא-דאטה: {}", e)))?
        };

        // עדכון המטריקות והזמן
        metadata.metrics = metrics;
        metadata.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // שמירת המטא-דאטה המעודכן
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בהמרת מטא-דאטה: {}", e)))?;
        
        fs::write(&metadata_path, metadata_json)
            .map_err(|e| TranslationError::LearningError(format!("שגיאה בשמירת מטא-דאטה: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_metrics() -> ModelMetrics {
        ModelMetrics {
            translation_accuracy: 0.85,
            bleu_score: 0.75,
            final_loss: 0.1,
            training_steps: 1000,
        }
    }

    fn create_test_config() -> ModelConfig {
        ModelConfig {
            vocab_size: 32000,
            num_layers: 6,
            model_size: 512,
            embedding_dim: 256,
            num_heads: 8,
        }
    }

    #[test]
    fn test_model_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModelStorage::new(temp_dir.path()).unwrap();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_save_and_load_model() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModelStorage::new(temp_dir.path()).unwrap();
        
        // יצירת קובץ מודל זמני
        let model_file = temp_dir.path().join("temp_model.pt");
        File::create(&model_file).unwrap();

        let metrics = create_test_metrics();
        let config = create_test_config();

        // שמירת המודל
        storage.save_model(
            "test_model",
            "Test model description",
            "1.0.0",
            &model_file,
            metrics.clone(),
            config.clone(),
        ).unwrap();

        // טעינת המודל
        let (loaded_path, metadata) = storage.load_model("test_model", "1.0.0").unwrap();
        
        assert!(loaded_path.exists());
        assert_eq!(metadata.name, "test_model");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.metrics.translation_accuracy, metrics.translation_accuracy);
    }

    #[test]
    fn test_list_models() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModelStorage::new(temp_dir.path()).unwrap();
        
        let model_file = temp_dir.path().join("temp_model.pt");
        File::create(&model_file).unwrap();

        // שמירת שני מודלים
        storage.save_model(
            "model1",
            "First model",
            "1.0.0",
            &model_file,
            create_test_metrics(),
            create_test_config(),
        ).unwrap();

        storage.save_model(
            "model2",
            "Second model",
            "1.0.0",
            &model_file,
            create_test_metrics(),
            create_test_config(),
        ).unwrap();

        let models = storage.list_models().unwrap();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_delete_model() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModelStorage::new(temp_dir.path()).unwrap();
        
        let model_file = temp_dir.path().join("temp_model.pt");
        File::create(&model_file).unwrap();

        // שמירת מודל
        storage.save_model(
            "test_model",
            "Test model",
            "1.0.0",
            &model_file,
            create_test_metrics(),
            create_test_config(),
        ).unwrap();

        // מחיקת המודל
        storage.delete_model("test_model", "1.0.0").unwrap();
        
        // וידוא שהמודל נמחק
        assert!(storage.load_model("test_model", "1.0.0").is_err());
    }

    #[test]
    fn test_update_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModelStorage::new(temp_dir.path()).unwrap();
        
        let model_file = temp_dir.path().join("temp_model.pt");
        File::create(&model_file).unwrap();

        // שמירת מודל
        storage.save_model(
            "test_model",
            "Test model",
            "1.0.0",
            &model_file,
            create_test_metrics(),
            create_test_config(),
        ).unwrap();

        // עדכון מטריקות
        let new_metrics = ModelMetrics {
            translation_accuracy: 0.9,
            bleu_score: 0.8,
            final_loss: 0.05,
            training_steps: 2000,
        };

        storage.update_metrics("test_model", "1.0.0", new_metrics.clone()).unwrap();

        // בדיקת העדכון
        let (_, metadata) = storage.load_model("test_model", "1.0.0").unwrap();
        assert_eq!(metadata.metrics.translation_accuracy, new_metrics.translation_accuracy);
        assert_eq!(metadata.metrics.training_steps, new_metrics.training_steps);
    }
} 