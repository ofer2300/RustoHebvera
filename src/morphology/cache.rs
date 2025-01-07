use super::*;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct CacheEntry<T> {
    data: T,
    timestamp: SystemTime,
    hits: u32,
}

impl<T> CacheEntry<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            timestamp: SystemTime::now(),
            hits: 1,
        }
    }

    fn update_hits(&mut self) {
        self.hits += 1;
        self.timestamp = SystemTime::now();
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed().unwrap_or_default() > ttl
    }
}

#[derive(Debug)]
pub struct MorphologyCache {
    hebrew_cache: RwLock<HashMap<String, CacheEntry<EnhancedHebrewMorphology>>>,
    russian_cache: RwLock<HashMap<String, CacheEntry<EnhancedRussianMorphology>>>,
    max_size: usize,
    ttl: Duration,
}

impl MorphologyCache {
    pub fn new() -> Self {
        Self {
            hebrew_cache: RwLock::new(HashMap::new()),
            russian_cache: RwLock::new(HashMap::new()),
            max_size: 10000, // גודל מקסימלי של המטמון
            ttl: Duration::from_secs(3600), // זמן תפוגה של שעה
        }
    }

    pub fn with_config(max_size: usize, ttl_secs: u64) -> Self {
        Self {
            hebrew_cache: RwLock::new(HashMap::new()),
            russian_cache: RwLock::new(HashMap::new()),
            max_size,
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get_hebrew(&self, word: &str) -> Option<EnhancedHebrewMorphology> {
        let mut cache = self.hebrew_cache.write().unwrap();
        
        if let Some(entry) = cache.get_mut(word) {
            if !entry.is_expired(self.ttl) {
                entry.update_hits();
                return Some(entry.data.clone());
            } else {
                cache.remove(word);
            }
        }
        
        None
    }

    pub fn get_russian(&self, word: &str) -> Option<EnhancedRussianMorphology> {
        let mut cache = self.russian_cache.write().unwrap();
        
        if let Some(entry) = cache.get_mut(word) {
            if !entry.is_expired(self.ttl) {
                entry.update_hits();
                return Some(entry.data.clone());
            } else {
                cache.remove(word);
            }
        }
        
        None
    }

    pub fn store_hebrew(&self, word: String, analysis: EnhancedHebrewMorphology) {
        let mut cache = self.hebrew_cache.write().unwrap();
        
        if cache.len() >= self.max_size {
            self.cleanup_hebrew_cache(&mut cache);
        }
        
        cache.insert(word, CacheEntry::new(analysis));
    }

    pub fn store_russian(&self, word: String, analysis: EnhancedRussianMorphology) {
        let mut cache = self.russian_cache.write().unwrap();
        
        if cache.len() >= self.max_size {
            self.cleanup_russian_cache(&mut cache);
        }
        
        cache.insert(word, CacheEntry::new(analysis));
    }

    fn cleanup_hebrew_cache(&self, cache: &mut HashMap<String, CacheEntry<EnhancedHebrewMorphology>>) {
        // מחיקת ערכים שפג תוקפם
        cache.retain(|_, entry| !entry.is_expired(self.ttl));
        
        if cache.len() >= self.max_size {
            // מחיקת הערכים הפחות שימושיים
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by_key(|(_, entry)| entry.hits);
            let to_remove = entries.len() - (self.max_size * 9 / 10); // משאיר 90% מהגודל המקסימלי
            
            for (word, _) in entries.iter().take(to_remove) {
                cache.remove(*word);
            }
        }
    }

    fn cleanup_russian_cache(&self, cache: &mut HashMap<String, CacheEntry<EnhancedRussianMorphology>>) {
        // מחיקת ערכים שפג תוקפם
        cache.retain(|_, entry| !entry.is_expired(self.ttl));
        
        if cache.len() >= self.max_size {
            // מחיקת הערכים הפחות שימושיים
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by_key(|(_, entry)| entry.hits);
            let to_remove = entries.len() - (self.max_size * 9 / 10); // משאיר 90% מהגודל המקסימלי
            
            for (word, _) in entries.iter().take(to_remove) {
                cache.remove(*word);
            }
        }
    }

    pub fn clear(&self) {
        self.hebrew_cache.write().unwrap().clear();
        self.russian_cache.write().unwrap().clear();
    }

    pub fn get_stats(&self) -> CacheStats {
        let hebrew = self.hebrew_cache.read().unwrap();
        let russian = self.russian_cache.read().unwrap();

        CacheStats {
            hebrew_entries: hebrew.len(),
            russian_entries: russian.len(),
            hebrew_hits: hebrew.values().map(|e| e.hits).sum(),
            russian_hits: russian.values().map(|e| e.hits).sum(),
            max_size: self.max_size,
            ttl_secs: self.ttl.as_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hebrew_entries: usize,
    pub russian_entries: usize,
    pub hebrew_hits: u32,
    pub russian_hits: u32,
    pub max_size: usize,
    pub ttl_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_cache_expiration() {
        let cache = MorphologyCache::with_config(100, 1); // TTL של שנייה אחת
        
        let analysis = EnhancedHebrewMorphology {
            basic: HebrewMorphology {
                root: vec!['כ', 'ת', 'ב'],
                pattern: None,
                binyan: None,
                gender: None,
                number: None,
            },
            root_analysis: None,
            tense: None,
            person: None,
            is_construct_state: false,
            semantic_info: None,
            confidence_score: 0.8,
        };
        
        cache.store_hebrew("test".to_string(), analysis.clone());
        assert!(cache.get_hebrew("test").is_some());
        
        sleep(Duration::from_secs(2));
        assert!(cache.get_hebrew("test").is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = MorphologyCache::with_config(2, 3600);
        
        let analysis = EnhancedHebrewMorphology {
            basic: HebrewMorphology {
                root: vec!['כ', 'ת', 'ב'],
                pattern: None,
                binyan: None,
                gender: None,
                number: None,
            },
            root_analysis: None,
            tense: None,
            person: None,
            is_construct_state: false,
            semantic_info: None,
            confidence_score: 0.8,
        };
        
        cache.store_hebrew("word1".to_string(), analysis.clone());
        cache.store_hebrew("word2".to_string(), analysis.clone());
        cache.store_hebrew("word3".to_string(), analysis.clone());
        
        let stats = cache.get_stats();
        assert!(stats.hebrew_entries <= 2);
    }
} 