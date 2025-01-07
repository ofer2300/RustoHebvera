use super::*;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use rayon::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub name: String,
    pub description: String,
    pub parent_domain: Option<String>,
    pub sub_domains: Vec<String>,
    pub keywords: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterInfo {
    pub level: String,
    pub description: String,
    pub typical_contexts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    pub text: String,
    pub domain: String,
    pub register: String,
    pub frequency: f32,
}

#[derive(Debug)]
pub struct SemanticAnalyzer {
    domain_database: HashMap<String, DomainInfo>,
    register_database: HashMap<String, RegisterInfo>,
    usage_database: HashMap<String, Vec<UsageExample>>,
    domain_index: HashMap<String, HashSet<String>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            domain_database: Self::load_domains(),
            register_database: Self::load_registers(),
            usage_database: Self::load_usage_examples(),
            domain_index: HashMap::new(),
        }
    }

    fn load_domains() -> HashMap<String, DomainInfo> {
        // כאן נטען את בסיס הנתונים של התחומים
        let mut domains = HashMap::new();
        
        // דוגמה לתחומים
        domains.insert("טכני".to_string(), DomainInfo {
            name: "טכני".to_string(),
            description: "מונחים טכניים והנדסיים".to_string(),
            parent_domain: None,
            sub_domains: vec!["מחשבים".to_string(), "אלקטרוניקה".to_string()],
            keywords: ["מכונה", "מערכת", "תהליך"].iter().map(|s| s.to_string()).collect(),
        });
        
        domains.insert("מחשבים".to_string(), DomainInfo {
            name: "מחשבים".to_string(),
            description: "מונחי מחשבים ותוכנה".to_string(),
            parent_domain: Some("טכני".to_string()),
            sub_domains: vec!["תכנות".to_string(), "רשתות".to_string()],
            keywords: ["תוכנה", "מחשב", "קוד"].iter().map(|s| s.to_string()).collect(),
        });
        
        domains
    }

    fn load_registers() -> HashMap<String, RegisterInfo> {
        // כאן נטען את בסיס הנתונים של המשלבים
        let mut registers = HashMap::new();
        
        registers.insert("טכני_גבוה".to_string(), RegisterInfo {
            level: "גבוה".to_string(),
            description: "משלב טכני פורמלי".to_string(),
            typical_contexts: vec!["מסמכים טכניים".to_string(), "תיעוד מקצועי".to_string()],
        });
        
        registers.insert("טכני_בינוני".to_string(), RegisterInfo {
            level: "בינוני".to_string(),
            description: "משלב טכני יומיומי".to_string(),
            typical_contexts: vec!["הוראות הפעלה".to_string(), "מדריכים למשתמש".to_string()],
        });
        
        registers
    }

    fn load_usage_examples() -> HashMap<String, Vec<UsageExample>> {
        // כאן נטען את בסיס הנתונים של דוגמאות השימוש
        HashMap::new()
    }

    pub fn analyze_context(&self, word: &str, context: Option<&str>) -> Option<SemanticInfo> {
        let domains = self.identify_domains(word, context);
        let register = self.identify_register(word, context);
        let examples = self.find_usage_examples(word, &domains);
        
        if domains.is_empty() && examples.is_empty() {
            return None;
        }
        
        Some(SemanticInfo {
            domain: domains,
            register,
            usage_examples: examples,
        })
    }

    fn identify_domains(&self, word: &str, context: Option<&str>) -> Vec<String> {
        let mut domains = HashSet::new();
        
        // בדיקת התאמה לפי מילות מפתח
        for (domain_name, info) in &self.domain_database {
            if info.keywords.contains(word) {
                domains.insert(domain_name.clone());
                // הוספת תחומי-על
                if let Some(parent) = &info.parent_domain {
                    domains.insert(parent.clone());
                }
            }
        }
        
        // בדיקת הקשר אם קיים
        if let Some(ctx) = context {
            for (domain_name, info) in &self.domain_database {
                if info.keywords.iter().any(|k| ctx.contains(k)) {
                    domains.insert(domain_name.clone());
                }
            }
        }
        
        domains.into_iter().collect()
    }

    fn identify_register(&self, word: &str, context: Option<&str>) -> String {
        // ברירת מחדל למשלב טכני בינוני
        "טכני_בינוני".to_string()
    }

    fn find_usage_examples(&self, word: &str, domains: &[String]) -> Vec<String> {
        self.usage_database.get(word)
            .map(|examples| {
                examples.iter()
                    .filter(|ex| domains.contains(&ex.domain))
                    .map(|ex| ex.text.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn add_domain(&mut self, domain: DomainInfo) {
        self.domain_database.insert(domain.name.clone(), domain);
        self.build_domain_index();
    }

    pub fn add_usage_example(&mut self, word: String, example: UsageExample) {
        self.usage_database
            .entry(word)
            .or_insert_with(Vec::new)
            .push(example);
    }

    fn build_domain_index(&mut self) {
        self.domain_index.clear();
        
        for (domain_name, info) in &self.domain_database {
            for keyword in &info.keywords {
                self.domain_index
                    .entry(keyword.clone())
                    .or_insert_with(HashSet::new)
                    .insert(domain_name.clone());
            }
        }
    }

    pub fn get_domain_hierarchy(&self, domain: &str) -> Option<Vec<String>> {
        let mut hierarchy = Vec::new();
        let mut current = Some(domain.to_string());
        
        while let Some(domain_name) = current {
            hierarchy.push(domain_name.clone());
            current = self.domain_database.get(&domain_name)
                .and_then(|info| info.parent_domain.clone());
        }
        
        if hierarchy.is_empty() {
            None
        } else {
            Some(hierarchy)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_identification() {
        let analyzer = SemanticAnalyzer::new();
        let context = Some("הגדרת פונקציה בשפת תכנות");
        let domains = analyzer.identify_domains("קוד", context);
        assert!(domains.contains(&"מחשבים".to_string()));
    }

    #[test]
    fn test_register_identification() {
        let analyzer = SemanticAnalyzer::new();
        let register = analyzer.identify_register("אלגוריתם", Some("תיעוד טכני של מערכת"));
        assert_eq!(register, "טכני_בינוני");
    }

    #[test]
    fn test_domain_hierarchy() {
        let analyzer = SemanticAnalyzer::new();
        let hierarchy = analyzer.get_domain_hierarchy("מחשבים");
        assert!(hierarchy.is_some());
        let hierarchy = hierarchy.unwrap();
        assert!(hierarchy.contains(&"טכני".to_string()));
    }

    #[test]
    fn test_add_domain() {
        let mut analyzer = SemanticAnalyzer::new();
        let domain = DomainInfo {
            name: "בינה_מלאכותית".to_string(),
            description: "מונחים בתחום הבינה המלאכותית".to_string(),
            parent_domain: Some("מחשבים".to_string()),
            sub_domains: vec![],
            keywords: ["למידה", "רשת_עצבית"].iter().map(|s| s.to_string()).collect(),
        };
        
        analyzer.add_domain(domain);
        let hierarchy = analyzer.get_domain_hierarchy("בינה_מלאכותית");
        assert!(hierarchy.is_some());
    }
} 