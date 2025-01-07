use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref HEBREW_LETTERS: HashSet<char> = {
        let mut set = HashSet::new();
        "אבגדהוזחטיכלמנסעפצקרשת".chars().for_each(|c| { set.insert(c); });
        set
    };

    static ref RUSSIAN_LETTERS: HashSet<char> = {
        let mut set = HashSet::new();
        "абвгдеёжзийклмнопрстуфхцчшщъыьэюя".chars().for_each(|c| { set.insert(c); });
        set
    };

    static ref HEBREW_PREFIXES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("ב");
        set.insert("ה");
        set.insert("ו");
        set.insert("כ");
        set.insert("ל");
        set.insert("מ");
        set.insert("ש");
        set
    };

    static ref HEBREW_SUFFIXES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("ים");
        set.insert("ות");
        set.insert("יים");
        set.insert("תיים");
        set.insert("ה");
        set.insert("ת");
        set.insert("י");
        set.insert("נו");
        set.insert("כם");
        set.insert("כן");
        set.insert("הם");
        set.insert("הן");
        set
    };
}

/// בודק אם תו הוא אות עברית
pub fn is_hebrew_letter(c: char) -> bool {
    HEBREW_LETTERS.contains(&c)
}

/// בודק אם תו הוא אות רוסית
pub fn is_russian_letter(c: char) -> bool {
    RUSSIAN_LETTERS.contains(&c)
}

/// בודק אם מילה היא בעברית
pub fn is_hebrew_word(word: &str) -> bool {
    word.chars().all(|c| is_hebrew_letter(c) || c.is_whitespace())
}

/// בודק אם מילה היא ברוסית
pub fn is_russian_word(word: &str) -> bool {
    word.chars().all(|c| is_russian_letter(c) || c.is_whitespace())
}

/// מסיר תחיליות מוכרות ממילה בעברית
pub fn remove_hebrew_prefixes(word: &str) -> String {
    let mut result = word.to_string();
    for prefix in HEBREW_PREFIXES.iter() {
        if result.starts_with(prefix) {
            result = result[prefix.len()..].to_string();
        }
    }
    result
}

/// מסיר סופיות מוכרות ממילה בעברית
pub fn remove_hebrew_suffixes(word: &str) -> String {
    let mut result = word.to_string();
    for suffix in HEBREW_SUFFIXES.iter() {
        if result.ends_with(suffix) {
            result = result[..result.len() - suffix.len()].to_string();
            break;
        }
    }
    result
}

/// מנקה מילה מניקוד וסימנים מיוחדים
pub fn clean_word(word: &str) -> String {
    word.chars()
        .filter(|c| !c.is_ascii_punctuation() && !c.is_ascii_whitespace())
        .collect()
}

/// מחלק טקסט למילים
pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| clean_word(word))
        .filter(|word| !word.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hebrew_word() {
        assert!(is_hebrew_word("שלום"));
        assert!(is_hebrew_word("בית"));
        assert!(!is_hebrew_word("hello"));
        assert!(!is_hebrew_word("привет"));
    }

    #[test]
    fn test_is_russian_word() {
        assert!(is_russian_word("привет"));
        assert!(is_russian_word("мир"));
        assert!(!is_russian_word("hello"));
        assert!(!is_russian_word("שלום"));
    }

    #[test]
    fn test_remove_hebrew_prefixes() {
        assert_eq!(remove_hebrew_prefixes("השלום"), "שלום");
        assert_eq!(remove_hebrew_prefixes("בבית"), "בית");
        assert_eq!(remove_hebrew_prefixes("לכתוב"), "כתוב");
    }

    #[test]
    fn test_remove_hebrew_suffixes() {
        assert_eq!(remove_hebrew_suffixes("ספרים"), "ספר");
        assert_eq!(remove_hebrew_suffixes("מחברות"), "מחבר");
        assert_eq!(remove_hebrew_suffixes("שולחנות"), "שולחן");
    }

    #[test]
    fn test_tokenize() {
        let text = "שלום עולם! מה נשמע?";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["שלום", "עולם", "מה", "נשמע"]);
    }
} 