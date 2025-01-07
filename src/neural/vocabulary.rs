use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, Result as IoResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VocabularyError {
    #[error("שגיאת קריאה/כתיבה: {0}")]
    Io(#[from] std::io::Error),
    #[error("מילה לא נמצאה במילון: {0}")]
    WordNotFound(String),
    #[error("אינדקס לא חוקי: {0}")]
    InvalidIndex(i64),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vocabulary {
    word_to_index: HashMap<String, i64>,
    index_to_word: HashMap<i64, String>,
    next_index: i64,
}

impl Vocabulary {
    pub fn new() -> Self {
        let mut vocab = Self {
            word_to_index: HashMap::new(),
            index_to_word: HashMap::new(),
            next_index: 0,
        };

        // הוספת טוקנים מיוחדים
        vocab.add_special_token("<PAD>");  // ריפוד למשפטים קצרים
        vocab.add_special_token("<UNK>");  // מילים לא מוכרות
        vocab.add_special_token("<BOS>");  // תחילת משפט
        vocab.add_special_token("<EOS>");  // סוף משפט
        
        vocab
    }

    fn add_special_token(&mut self, token: &str) {
        self.add_word(token);
    }

    pub fn add_word(&mut self, word: &str) -> i64 {
        if let Some(&idx) = self.word_to_index.get(word) {
            return idx;
        }

        let idx = self.next_index;
        self.word_to_index.insert(word.to_string(), idx);
        self.index_to_word.insert(idx, word.to_string());
        self.next_index += 1;
        idx
    }

    pub fn get_index(&self, word: &str) -> Result<i64, VocabularyError> {
        self.word_to_index
            .get(word)
            .copied()
            .ok_or_else(|| VocabularyError::WordNotFound(word.to_string()))
    }

    pub fn get_word(&self, idx: i64) -> Result<String, VocabularyError> {
        self.index_to_word
            .get(&idx)
            .cloned()
            .ok_or_else(|| VocabularyError::InvalidIndex(idx))
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), VocabularyError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let word = line?;
            self.add_word(&word);
        }

        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), VocabularyError> {
        let mut file = File::create(path)?;
        
        let mut words: Vec<_> = self.word_to_index.iter().collect();
        words.sort_by_key(|(_, &idx)| idx);

        for (word, _) in words {
            writeln!(file, "{}", word)?;
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.word_to_index.len()
    }

    pub fn contains(&self, word: &str) -> bool {
        self.word_to_index.contains_key(word)
    }

    pub fn get_unk_index(&self) -> i64 {
        self.word_to_index["<UNK>"]
    }

    pub fn get_pad_index(&self) -> i64 {
        self.word_to_index["<PAD>"]
    }

    pub fn get_bos_index(&self) -> i64 {
        self.word_to_index["<BOS>"]
    }

    pub fn get_eos_index(&self) -> i64 {
        self.word_to_index["<EOS>"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_vocabulary() {
        let vocab = Vocabulary::new();
        assert!(vocab.contains("<PAD>"));
        assert!(vocab.contains("<UNK>"));
        assert!(vocab.contains("<BOS>"));
        assert!(vocab.contains("<EOS>"));
    }

    #[test]
    fn test_add_and_get_word() {
        let mut vocab = Vocabulary::new();
        let idx = vocab.add_word("שלום");
        assert_eq!(vocab.get_index("שלום").unwrap(), idx);
        assert_eq!(vocab.get_word(idx).unwrap(), "שלום");
    }

    #[test]
    fn test_unknown_word() {
        let vocab = Vocabulary::new();
        assert!(matches!(
            vocab.get_index("לא-קיים"),
            Err(VocabularyError::WordNotFound(_))
        ));
    }

    #[test]
    fn test_invalid_index() {
        let vocab = Vocabulary::new();
        assert!(matches!(
            vocab.get_word(999),
            Err(VocabularyError::InvalidIndex(_))
        ));
    }

    #[test]
    fn test_save_and_load() -> Result<(), VocabularyError> {
        let mut vocab = Vocabulary::new();
        vocab.add_word("שלום");
        vocab.add_word("עולם");

        let mut temp_file = NamedTempFile::new().unwrap();
        vocab.save_to_file(temp_file.path())?;

        let mut new_vocab = Vocabulary::new();
        new_vocab.load_from_file(temp_file.path())?;

        assert_eq!(vocab.size(), new_vocab.size());
        assert_eq!(
            vocab.get_index("שלום").unwrap(),
            new_vocab.get_index("שלום").unwrap()
        );
        assert_eq!(
            vocab.get_index("עולם").unwrap(),
            new_vocab.get_index("עולם").unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_special_tokens() {
        let vocab = Vocabulary::new();
        assert!(vocab.get_pad_index() >= 0);
        assert!(vocab.get_unk_index() >= 0);
        assert!(vocab.get_bos_index() >= 0);
        assert!(vocab.get_eos_index() >= 0);
    }
} 