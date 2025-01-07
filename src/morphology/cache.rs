use std::collections::HashMap;
use std::sync::RwLock;
use super::{MorphologyAnalysis, MorphologyError};

#[derive(Debug)]
pub struct MorphologyCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
    max_size: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    analysis: MorphologyAnalysis,
    timestamp: std::time::SystemTime,
    hits: u32,
}

impl MorphologyCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size: 10000,
        }
    }

    pub fn get(&self, key: &str) -> Option<MorphologyAnalysis> {
        let cache = self.cache.read().unwrap();
        cache.get(key).map(|entry| {
            let mut cache = self.cache.write().unwrap();
            if let Some(entry) = cache.get_mut(key) {
                entry.hits += 1;
            }
            entry.analysis.clone()
        })
    }

    pub fn store(&self, key: String, analysis: MorphologyAnalysis) -> Result<(), MorphologyError> {
        let mut cache = self.cache.write().unwrap();
        
        if cache.len() >= self.max_size {
            self.cleanup(&mut cache);
        }

        cache.insert(key, CacheEntry {
            analysis,
            timestamp: std::time::SystemTime::now(),
            hits: 1,
        });

        Ok(())
    }

    fn cleanup(&self, cache: &mut HashMap<String, CacheEntry>) {
        let now = std::time::SystemTime::now();
        let one_hour = std::time::Duration::from_secs(3600);

        cache.retain(|_, entry| {
            entry.timestamp.elapsed().unwrap() < one_hour
        });
    }
} 