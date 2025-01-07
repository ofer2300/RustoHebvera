use std::path::Path;
use anyhow::{Result, Context};
use font_kit::source::SystemSource;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::handle::Handle;
use std::sync::Arc;
use printpdf::{PdfDocument, IndirectFontRef};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use base64::Engine;

#[derive(Debug, Clone)]
pub struct FontData {
    pub name: String,
    pub data: Vec<u8>,
    pub properties: FontProperties,
}

#[derive(Debug, Clone)]
pub struct FontProperties {
    pub is_bold: bool,
    pub is_italic: bool,
    pub supports_hebrew: bool,
    pub supports_russian: bool,
}

pub struct FontManager {
    fonts: HashMap<String, FontData>,
    system_source: SystemSource,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            system_source: SystemSource::new(),
        }
    }

    pub fn load_system_fonts(&mut self) -> Result<()> {
        // טעינת פונטים עבריים
        self.load_font_family("David CLM", true, false)?;
        self.load_font_family("Frank Ruehl CLM", true, false)?;
        self.load_font_family("Miriam CLM", true, false)?;
        
        // טעינת פונטים רוסיים
        self.load_font_family("Times New Roman", false, true)?;
        self.load_font_family("Arial", false, true)?;
        
        // טעינת פונטים דו-לשוניים
        self.load_font_family("Noto Sans Hebrew", true, true)?;
        self.load_font_family("Open Sans Hebrew", true, true)?;
        
        Ok(())
    }

    fn load_font_family(&mut self, family_name: &str, hebrew: bool, russian: bool) -> Result<()> {
        let family = self.system_source.select_family_by_name(family_name)?;
        
        for properties in [
            Properties::new(),
            Properties {
                weight: font_kit::properties::Weight::BOLD,
                ..Properties::new()
            },
            Properties {
                style: font_kit::properties::Style::Italic,
                ..Properties::new()
            },
        ] {
            if let Ok(handle) = family.select_best_match(&[properties.clone()]) {
                let font_data = match handle {
                    Handle::Memory { bytes, .. } => bytes.to_vec(),
                    Handle::Path { path, .. } => {
                        let mut file = File::open(path)?;
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer)?;
                        buffer
                    }
                };
                
                let font_properties = FontProperties {
                    is_bold: properties.weight == font_kit::properties::Weight::BOLD,
                    is_italic: properties.style == font_kit::properties::Style::Italic,
                    supports_hebrew: hebrew,
                    supports_russian: russian,
                };
                
                let font_name = format!("{}-{}-{}",
                    family_name,
                    if font_properties.is_bold { "Bold" } else { "Regular" },
                    if font_properties.is_italic { "Italic" } else { "Normal" }
                );
                
                self.fonts.insert(font_name.clone(), FontData {
                    name: font_name,
                    data: font_data,
                    properties: font_properties,
                });
            }
        }
        
        Ok(())
    }

    pub fn get_font_for_text(&self, text: &str) -> Option<&FontData> {
        let has_hebrew = text.chars().any(|c| is_hebrew_char(c));
        let has_russian = text.chars().any(|c| is_russian_char(c));
        
        // בחירת פונט מתאים לפי השפות בטקסט
        self.fonts.values().find(|font| {
            (has_hebrew && font.properties.supports_hebrew) ||
            (has_russian && font.properties.supports_russian) ||
            (!has_hebrew && !has_russian)
        })
    }

    pub fn embed_font(&self, font: &FontData, doc: &PdfDocument) -> Result<IndirectFontRef> {
        let font_ref = doc.add_external_font(&font.data)?;
        Ok(font_ref)
    }
}

fn is_hebrew_char(c: char) -> bool {
    matches!(c as u32, 0x0590..=0x05FF)
}

fn is_russian_char(c: char) -> bool {
    matches!(c as u32, 0x0400..=0x04FF)
} 