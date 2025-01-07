use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalTerm {
    pub hebrew: String,
    pub russian: String,
    pub context: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
    pub synonyms_he: Vec<String>,
    pub synonyms_ru: Vec<String>,
    pub usage_examples: Vec<String>,
    pub tags: HashSet<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub lang: String,
    pub categories: Option<Vec<String>>,
    pub contexts: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub include_synonyms: bool,
    pub exact_match: bool,
}

#[derive(Debug)]
pub struct TechnicalDictionary {
    terms: HashMap<String, TechnicalTerm>,
    file_path: String,
    category_index: HashMap<String, HashSet<String>>,  // קטגוריה -> רשימת מונחים
    context_index: HashMap<String, HashSet<String>>,   // הקשר -> רשימת מונחים
    tag_index: HashMap<String, HashSet<String>>,       // תגית -> רשימת מונחים
}

impl TechnicalDictionary {
    pub fn new(file_path: String) -> Result<Self> {
        let terms = if Path::new(&file_path).exists() {
            let content = fs::read_to_string(&file_path)?;
            serde_json::from_str(&content)?
        } else {
            HashMap::new()
        };

        let mut dict = Self {
            terms,
            file_path,
            category_index: HashMap::new(),
            context_index: HashMap::new(),
            tag_index: HashMap::new(),
        };
        
        dict.rebuild_indices();
        Ok(dict)
    }

    fn rebuild_indices(&mut self) {
        self.category_index.clear();
        self.context_index.clear();
        self.tag_index.clear();

        for (term_key, term) in &self.terms {
            if let Some(category) = &term.category {
                self.category_index
                    .entry(category.clone())
                    .or_default()
                    .insert(term_key.clone());
            }
            
            if let Some(context) = &term.context {
                self.context_index
                    .entry(context.clone())
                    .or_default()
                    .insert(term_key.clone());
            }
            
            for tag in &term.tags {
                self.tag_index
                    .entry(tag.clone())
                    .or_default()
                    .insert(term_key.clone());
            }
        }
    }

    pub fn add_term(&mut self, mut term: TechnicalTerm) -> Result<()> {
        term.last_updated = chrono::Utc::now();
        self.terms.insert(term.hebrew.clone(), term);
        self.rebuild_indices();
        self.save()
    }

    pub fn update_term(&mut self, hebrew: &str, mut updates: TechnicalTerm) -> Result<()> {
        if let Some(term) = self.terms.get_mut(hebrew) {
            updates.last_updated = chrono::Utc::now();
            *term = updates;
            self.rebuild_indices();
            self.save()?;
        }
        Ok(())
    }

    pub fn delete_term(&mut self, hebrew: &str) -> Result<()> {
        self.terms.remove(hebrew);
        self.rebuild_indices();
        self.save()
    }

    pub fn search(&self, query: &SearchQuery) -> Vec<&TechnicalTerm> {
        let mut results: HashSet<&TechnicalTerm> = HashSet::new();
        let search_text = query.text.to_lowercase();

        // חיפוש בטקסט
        for term in self.terms.values() {
            let mut should_include = false;

            // חיפוש בשדה העיקרי
            let term_text = match query.lang.as_str() {
                "he" => &term.hebrew,
                "ru" => &term.russian,
                _ => continue,
            };

            if query.exact_match {
                should_include = term_text.to_lowercase() == search_text;
            } else {
                should_include = term_text.to_lowercase().contains(&search_text);
            }

            // חיפוש בסינונימים
            if query.include_synonyms {
                let synonyms = match query.lang.as_str() {
                    "he" => &term.synonyms_he,
                    "ru" => &term.synonyms_ru,
                    _ => continue,
                };
                
                should_include = should_include || synonyms.iter().any(|s| {
                    if query.exact_match {
                        s.to_lowercase() == search_text
                    } else {
                        s.to_lowercase().contains(&search_text)
                    }
                });
            }

            if should_include {
                results.insert(term);
            }
        }

        // סינון לפי קטגוריות
        if let Some(categories) = &query.categories {
            results.retain(|term| {
                if let Some(term_category) = &term.category {
                    categories.contains(term_category)
                } else {
                    false
                }
            });
        }

        // סינון לפי הקשרים
        if let Some(contexts) = &query.contexts {
            results.retain(|term| {
                if let Some(term_context) = &term.context {
                    contexts.contains(term_context)
                } else {
                    false
                }
            });
        }

        // סינון לפי תגיות
        if let Some(tags) = &query.tags {
            results.retain(|term| {
                tags.iter().all(|tag| term.tags.contains(tag))
            });
        }

        let mut results_vec: Vec<&TechnicalTerm> = results.into_iter().collect();
        results_vec.sort_by(|a, b| a.hebrew.cmp(&b.hebrew));
        results_vec
    }

    pub fn get_term(&self, hebrew: &str) -> Option<&TechnicalTerm> {
        self.terms.get(hebrew)
    }

    pub fn get_all_terms(&self) -> impl Iterator<Item = &TechnicalTerm> {
        self.terms.values()
    }

    pub fn get_all_categories(&self) -> Vec<String> {
        self.category_index.keys().cloned().collect()
    }

    pub fn get_all_contexts(&self) -> Vec<String> {
        self.context_index.keys().cloned().collect()
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        self.tag_index.keys().cloned().collect()
    }

    pub fn get_terms_by_category(&self, category: &str) -> Vec<&TechnicalTerm> {
        self.category_index.get(category)
            .map(|terms| {
                terms.iter()
                    .filter_map(|term_key| self.terms.get(term_key))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_terms_by_context(&self, context: &str) -> Vec<&TechnicalTerm> {
        self.context_index.get(context)
            .map(|terms| {
                terms.iter()
                    .filter_map(|term_key| self.terms.get(term_key))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_terms_by_tag(&self, tag: &str) -> Vec<&TechnicalTerm> {
        self.tag_index.get(tag)
            .map(|terms| {
                terms.iter()
                    .filter_map(|term_key| self.terms.get(term_key))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> String {
        let mut result = text.to_string();
        
        for term in self.terms.values() {
            let (search_term, replacement) = match source_lang {
                "he" => (&term.hebrew, &term.russian),
                "ru" => (&term.russian, &term.hebrew),
                _ => continue,
            };
            
            result = result.replace(search_term, replacement);
            
            // חיפוש בסינונימים
            let synonyms = match source_lang {
                "he" => &term.synonyms_he,
                "ru" => &term.synonyms_ru,
                _ => continue,
            };
            
            for synonym in synonyms {
                result = result.replace(synonym, replacement);
            }
        }
        
        result
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.terms)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn get_all_terms_map(&self) -> Result<HashMap<String, TechnicalTerm>> {
        Ok(self.terms.clone())
    }

    pub fn get_all_tags_set(&self) -> Result<HashSet<String>> {
        let mut all_tags = HashSet::new();
        for term in self.terms.values() {
            all_tags.extend(term.tags.clone());
        }
        Ok(all_tags)
    }

    pub fn import_terms(&mut self, terms: HashMap<String, TechnicalTerm>) -> Result<()> {
        self.terms = terms;
        self.rebuild_indices();
        self.save()
    }

    pub fn merge_with(&mut self, other: &TechnicalDictionary) -> Result<()> {
        for (term_id, term) in &other.terms {
            if let Some(existing_term) = self.terms.get(term_id) {
                if existing_term.last_updated < term.last_updated {
                    self.terms.insert(term_id.clone(), term.clone());
                }
            } else {
                self.terms.insert(term_id.clone(), term.clone());
            }
        }
        self.rebuild_indices();
        self.save()
    }

    pub fn create_snapshot(&self) -> Result<HashMap<String, TechnicalTerm>> {
        Ok(self.terms.clone())
    }

    pub fn restore_snapshot(&mut self, snapshot: HashMap<String, TechnicalTerm>) -> Result<()> {
        self.terms = snapshot;
        self.rebuild_indices();
        self.save()
    }
} 