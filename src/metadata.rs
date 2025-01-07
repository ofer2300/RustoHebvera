use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    // מידע בסיסי
    pub title: String,
    pub author: String,
    pub created_date: DateTime<Utc>,
    pub modified_date: DateTime<Utc>,
    pub version: String,
    
    // מידע על השפה
    pub source_language: String,
    pub target_language: String,
    pub contains_technical_terms: bool,
    
    // מידע על התקנים
    pub standards: Vec<String>,
    pub certification_required: bool,
    
    // מידע על המסמך
    pub document_type: DocumentType,
    pub security_level: SecurityLevel,
    pub review_status: ReviewStatus,
    
    // תגיות ומידע נוסף
    pub tags: Vec<String>,
    pub custom_properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Technical,
    Standard,
    Drawing,
    Calculation,
    Manual,
    Report,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewStatus {
    Draft,
    InReview,
    Approved,
    Published,
    Archived,
}

impl DocumentMetadata {
    pub fn new(title: String, author: String, doc_type: DocumentType) -> Self {
        let now = Utc::now();
        Self {
            title,
            author,
            created_date: now,
            modified_date: now,
            version: "1.0.0".to_string(),
            source_language: "he".to_string(),
            target_language: "ru".to_string(),
            contains_technical_terms: false,
            standards: Vec::new(),
            certification_required: false,
            document_type: doc_type,
            security_level: SecurityLevel::Internal,
            review_status: ReviewStatus::Draft,
            tags: Vec::new(),
            custom_properties: HashMap::new(),
        }
    }

    pub fn update_modified_date(&mut self) {
        self.modified_date = Utc::now();
    }

    pub fn add_standard(&mut self, standard: String) {
        if !self.standards.contains(&standard) {
            self.standards.push(standard);
            self.update_modified_date();
        }
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.update_modified_date();
        }
    }

    pub fn set_custom_property(&mut self, key: String, value: String) {
        self.custom_properties.insert(key, value);
        self.update_modified_date();
    }

    pub fn increment_version(&mut self) {
        let version_parts: Vec<u32> = self.version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        if version_parts.len() == 3 {
            self.version = format!("{}.{}.{}",
                version_parts[0],
                version_parts[1],
                version_parts[2] + 1
            );
            self.update_modified_date();
        }
    }
}

pub trait MetadataProcessor {
    fn read_metadata(&self) -> Result<DocumentMetadata>;
    fn write_metadata(&mut self, metadata: &DocumentMetadata) -> Result<()>;
    fn update_metadata(&mut self, updates: HashMap<String, String>) -> Result<()>;
}

// מעבד מטא-דאטה ל-PDF
pub struct PdfMetadataProcessor {
    metadata: DocumentMetadata,
}

impl PdfMetadataProcessor {
    pub fn new(metadata: DocumentMetadata) -> Self {
        Self { metadata }
    }
}

impl MetadataProcessor for PdfMetadataProcessor {
    fn read_metadata(&self) -> Result<DocumentMetadata> {
        Ok(self.metadata.clone())
    }

    fn write_metadata(&mut self, metadata: &DocumentMetadata) -> Result<()> {
        self.metadata = metadata.clone();
        Ok(())
    }

    fn update_metadata(&mut self, updates: HashMap<String, String>) -> Result<()> {
        for (key, value) in updates {
            match key.as_str() {
                "title" => self.metadata.title = value,
                "author" => self.metadata.author = value,
                "version" => self.metadata.version = value,
                _ => { self.metadata.set_custom_property(key, value); }
            }
        }
        self.metadata.update_modified_date();
        Ok(())
    }
}

// מעבד מטא-דאטה ל-DOCX
pub struct DocxMetadataProcessor {
    metadata: DocumentMetadata,
}

impl DocxMetadataProcessor {
    pub fn new(metadata: DocumentMetadata) -> Self {
        Self { metadata }
    }
}

impl MetadataProcessor for DocxMetadataProcessor {
    fn read_metadata(&self) -> Result<DocumentMetadata> {
        Ok(self.metadata.clone())
    }

    fn write_metadata(&mut self, metadata: &DocumentMetadata) -> Result<()> {
        self.metadata = metadata.clone();
        Ok(())
    }

    fn update_metadata(&mut self, updates: HashMap<String, String>) -> Result<()> {
        for (key, value) in updates {
            match key.as_str() {
                "title" => self.metadata.title = value,
                "author" => self.metadata.author = value,
                "version" => self.metadata.version = value,
                _ => { self.metadata.set_custom_property(key, value); }
            }
        }
        self.metadata.update_modified_date();
        Ok(())
    }
}

// מעבד מטא-דאטה ל-Excel
pub struct ExcelMetadataProcessor {
    metadata: DocumentMetadata,
}

impl ExcelMetadataProcessor {
    pub fn new(metadata: DocumentMetadata) -> Self {
        Self { metadata }
    }
}

impl MetadataProcessor for ExcelMetadataProcessor {
    fn read_metadata(&self) -> Result<DocumentMetadata> {
        Ok(self.metadata.clone())
    }

    fn write_metadata(&mut self, metadata: &DocumentMetadata) -> Result<()> {
        self.metadata = metadata.clone();
        Ok(())
    }

    fn update_metadata(&mut self, updates: HashMap<String, String>) -> Result<()> {
        for (key, value) in updates {
            match key.as_str() {
                "title" => self.metadata.title = value,
                "author" => self.metadata.author = value,
                "version" => self.metadata.version = value,
                _ => { self.metadata.set_custom_property(key, value); }
            }
        }
        self.metadata.update_modified_date();
        Ok(())
    }
} 