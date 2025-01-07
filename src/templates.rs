use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use crate::metadata::{DocumentMetadata, DocumentType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    // מידע בסיסי על התבנית
    pub name: String,
    pub description: String,
    pub version: String,
    pub template_type: TemplateType,
    
    // מבנה המסמך
    pub sections: Vec<TemplateSection>,
    pub placeholders: HashMap<String, PlaceholderType>,
    pub styles: DocumentStyles,
    
    // הגדרות שפה
    pub rtl: bool,
    pub default_language: String,
    pub supported_languages: Vec<String>,
    
    // מטא-דאטה ברירת מחדל
    pub default_metadata: Option<DocumentMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateType {
    TechnicalSpec,
    StandardReport,
    DrawingSheet,
    CalculationSheet,
    Inspection,
    Approval,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSection {
    pub id: String,
    pub title: String,
    pub content: String,
    pub required: bool,
    pub order: u32,
    pub style: String,
    pub subsections: Vec<TemplateSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaceholderType {
    Text,
    Number,
    Date,
    List,
    Table,
    StandardReference,
    TechnicalTerm,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStyles {
    pub fonts: HashMap<String, FontStyle>,
    pub colors: HashMap<String, String>,
    pub spacing: SpacingStyle,
    pub page_layout: PageLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontStyle {
    pub family: String,
    pub size: f32,
    pub weight: String,
    pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingStyle {
    pub line_spacing: f32,
    pub paragraph_spacing: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLayout {
    pub size: String,
    pub orientation: String,
    pub margins: SpacingStyle,
    pub header_height: f32,
    pub footer_height: f32,
}

pub struct TemplateManager {
    templates: HashMap<String, DocumentTemplate>,
    template_dir: String,
}

impl TemplateManager {
    pub fn new(template_dir: String) -> Result<Self> {
        let mut manager = Self {
            templates: HashMap::new(),
            template_dir,
        };
        manager.load_templates()?;
        Ok(manager)
    }

    pub fn load_templates(&mut self) -> Result<()> {
        let template_path = Path::new(&self.template_dir);
        if !template_path.exists() {
            fs::create_dir_all(template_path)?;
            self.create_default_templates()?;
        }

        for entry in fs::read_dir(template_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let template: DocumentTemplate = serde_json::from_str(
                    &fs::read_to_string(entry.path())?
                )?;
                self.templates.insert(template.name.clone(), template);
            }
        }

        Ok(())
    }

    fn create_default_templates(&self) -> Result<()> {
        // תבנית למפרט טכני
        let tech_spec = DocumentTemplate {
            name: "technical_specification".to_string(),
            description: "תבנית למפרט טכני של מערכת כיבוי אש".to_string(),
            version: "1.0.0".to_string(),
            template_type: TemplateType::TechnicalSpec,
            sections: vec![
                TemplateSection {
                    id: "general".to_string(),
                    title: "כללי".to_string(),
                    content: "{{project_description}}".to_string(),
                    required: true,
                    order: 1,
                    style: "heading1".to_string(),
                    subsections: vec![],
                },
                TemplateSection {
                    id: "requirements".to_string(),
                    title: "דרישות מערכת".to_string(),
                    content: "{{system_requirements}}".to_string(),
                    required: true,
                    order: 2,
                    style: "heading1".to_string(),
                    subsections: vec![],
                },
            ],
            placeholders: {
                let mut map = HashMap::new();
                map.insert("project_description".to_string(), PlaceholderType::Text);
                map.insert("system_requirements".to_string(), PlaceholderType::Text);
                map
            },
            styles: DocumentStyles {
                fonts: {
                    let mut map = HashMap::new();
                    map.insert("default".to_string(), FontStyle {
                        family: "David CLM".to_string(),
                        size: 12.0,
                        weight: "normal".to_string(),
                        style: "normal".to_string(),
                    });
                    map
                },
                colors: {
                    let mut map = HashMap::new();
                    map.insert("text".to_string(), "#000000".to_string());
                    map.insert("heading".to_string(), "#333333".to_string());
                    map
                },
                spacing: SpacingStyle {
                    line_spacing: 1.5,
                    paragraph_spacing: 1.0,
                    margin_top: 20.0,
                    margin_bottom: 20.0,
                    margin_left: 25.0,
                    margin_right: 25.0,
                },
                page_layout: PageLayout {
                    size: "A4".to_string(),
                    orientation: "portrait".to_string(),
                    margins: SpacingStyle {
                        line_spacing: 1.0,
                        paragraph_spacing: 1.0,
                        margin_top: 25.4,
                        margin_bottom: 25.4,
                        margin_left: 25.4,
                        margin_right: 25.4,
                    },
                    header_height: 12.7,
                    footer_height: 12.7,
                },
            },
            rtl: true,
            default_language: "he".to_string(),
            supported_languages: vec!["he".to_string(), "ru".to_string()],
            default_metadata: None,
        };

        let template_path = Path::new(&self.template_dir)
            .join("technical_specification.json");
        fs::write(
            template_path,
            serde_json::to_string_pretty(&tech_spec)?
        )?;

        Ok(())
    }

    pub fn get_template(&self, name: &str) -> Option<&DocumentTemplate> {
        self.templates.get(name)
    }

    pub fn create_document_from_template(
        &self,
        template_name: &str,
        values: &HashMap<String, String>,
        metadata: Option<DocumentMetadata>,
    ) -> Result<String> {
        let template = self.get_template(template_name)
            .ok_or_else(|| anyhow::anyhow!("תבנית לא נמצאה: {}", template_name))?;
        
        let mut content = String::new();
        
        for section in &template.sections {
            // הוספת כותרת
            content.push_str(&format!("# {}\n\n", section.title));
            
            // עיבוד תוכן עם placeholders
            let mut section_content = section.content.clone();
            for (key, value) in values {
                section_content = section_content.replace(
                    &format!("{{{{{}}}}}", key),
                    value
                );
            }
            
            content.push_str(&section_content);
            content.push_str("\n\n");
            
            // עיבוד תתי-סעיפים
            for subsection in &section.subsections {
                content.push_str(&format!("## {}\n\n", subsection.title));
                let mut subsection_content = subsection.content.clone();
                for (key, value) in values {
                    subsection_content = subsection_content.replace(
                        &format!("{{{{{}}}}}", key),
                        value
                    );
                }
                content.push_str(&subsection_content);
                content.push_str("\n\n");
            }
        }
        
        Ok(content)
    }
} 