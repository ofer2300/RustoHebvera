use std::path::Path;
use anyhow::{Result, Context};
use std::fs::File;
use std::io::{Read, Write};
use encoding_rs::WINDOWS_1255;
use lopdf::{Document, Object, Dictionary, Stream};
use msoffice_docx::document::Document as DocxDocument;
use tempfile::NamedTempFile;
use calamine::{Reader, open_workbook, Xlsx, DataType};
use xlsxwriter::{Workbook, Worksheet};
use printpdf::{PdfDocument, PdfDocumentReference, Mm, Point, Rgb};
use rusttype::{Font, Scale};
use std::collections::HashMap;
use unic_bidi::BidiInfo;
use crate::fonts::{FontManager, FontData};
use crate::metadata::{DocumentMetadata, DocumentType, MetadataProcessor};
use chrono::Utc;
use uuid::Uuid;
use xmp::XmpMeta;

const DEFAULT_FONT_SIZE: f64 = 12.0;
const PAGE_WIDTH_MM: f64 = 210.0;  // A4
const PAGE_HEIGHT_MM: f64 = 297.0;  // A4
const MARGIN_MM: f64 = 20.0;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    PDF,
    DOCX,
    TXT,
    XLSX,
    CSV,
    Unknown,
}

#[derive(Debug)]
pub struct ExcelCell {
    pub value: String,
    pub row: u32,
    pub col: u32,
    pub is_formula: bool,
}

#[derive(Debug)]
pub struct ExcelSheet {
    pub name: String,
    pub cells: Vec<ExcelCell>,
}

pub struct FileProcessor {
    font_manager: FontManager,
    metadata: Option<DocumentMetadata>,
}

impl FileProcessor {
    pub fn new() -> Result<Self> {
        let mut font_manager = FontManager::new();
        font_manager.load_system_fonts()?;
        Ok(Self {
            font_manager,
            metadata: None,
        })
    }

    pub fn set_metadata(&mut self, metadata: DocumentMetadata) {
        self.metadata = Some(metadata);
    }

    pub fn get_metadata(&self) -> Option<&DocumentMetadata> {
        self.metadata.as_ref()
    }

    pub fn detect_file_type<P: AsRef<Path>>(path: P) -> FileType {
        match path.as_ref().extension().and_then(|e| e.to_str()) {
            Some("pdf") => FileType::PDF,
            Some("docx") => FileType::DOCX,
            Some("txt") => FileType::TXT,
            Some("xlsx") | Some("xls") => FileType::XLSX,
            Some("csv") => FileType::CSV,
            _ => FileType::Unknown,
        }
    }

    pub fn read_pdf<P: AsRef<Path>>(&self, path: P) -> Result<(String, Option<DocumentMetadata>)> {
        let doc = Document::load(path)
            .context("טעינת קובץ PDF נכשלה")?;
        
        // קריאת טקסט
        let mut text = String::new();
        for page_num in 1..=doc.get_pages().len() {
            if let Ok(content) = doc.extract_text(&[page_num]) {
                text.push_str(&content);
                text.push('\n');
            }
        }
        
        // קריאת מטא-דאטה
        let metadata = if let Some(info) = doc.get_info() {
            let mut meta = DocumentMetadata::new(
                info.get("Title").and_then(|o| o.as_string()).unwrap_or_default().to_string(),
                info.get("Author").and_then(|o| o.as_string()).unwrap_or_default().to_string(),
                DocumentType::Technical
            );
            
            if let Some(xmp_data) = doc.get_xmp_metadata() {
                if let Ok(xmp) = XmpMeta::from_str(&xmp_data) {
                    // קריאת מטא-דאטה נוספת מ-XMP
                    if let Some(lang) = xmp.get_property("dc:language") {
                        meta.set_custom_property("language".to_string(), lang);
                    }
                }
            }
            
            Some(meta)
        } else {
            None
        };
        
        Ok((text, metadata))
    }

    pub fn write_pdf(&self, text: &str, path: &Path) -> Result<()> {
        let (doc, page1, layer1) = PdfDocument::new(
            self.metadata.as_ref().map(|m| m.title.as_str()).unwrap_or("מסמך מתורגם"),
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            "Layer 1"
        );
        
        // כתיבת מטא-דאטה
        if let Some(metadata) = &self.metadata {
            doc.set_title(&metadata.title);
            doc.set_author(&metadata.author);
            doc.set_creation_date(metadata.created_date.timestamp() as i64);
            doc.set_modification_date(metadata.modified_date.timestamp() as i64);
            
            // יצירת XMP מטא-דאטה
            let mut xmp = XmpMeta::new();
            xmp.set_property("dc:title", &metadata.title);
            xmp.set_property("dc:creator", &metadata.author);
            xmp.set_property("dc:language", &metadata.source_language);
            
            doc.set_xmp_metadata(&xmp.to_string());
        }
        
        // כתיבת תוכן המסמך
        let current_layer = doc.get_page(page1).get_layer(layer1);
        let paragraphs = text.split('\n').collect::<Vec<_>>();
        
        let mut y_position = PAGE_HEIGHT_MM - MARGIN_MM;
        
        for paragraph in paragraphs {
            if paragraph.trim().is_empty() {
                y_position -= DEFAULT_FONT_SIZE * 1.5;
                continue;
            }
            
            let bidi_info = BidiInfo::new(paragraph, None);
            let paragraph_dir = if bidi_info.paragraphs[0].level.is_rtl() {
                TextDirection::RightToLeft
            } else {
                TextDirection::LeftToRight
            };
            
            let font = self.font_manager.get_font_for_text(paragraph)
                .ok_or_else(|| anyhow::anyhow!("לא נמצא פונט מתאים"))?;
            
            let font_ref = self.font_manager.embed_font(font, &doc)?;
            
            current_layer.begin_text_section();
            current_layer.set_font(&font_ref, DEFAULT_FONT_SIZE);
            
            match paragraph_dir {
                TextDirection::RightToLeft => {
                    current_layer.set_text_cursor(
                        Mm(PAGE_WIDTH_MM - MARGIN_MM),
                        Mm(y_position)
                    );
                    current_layer.set_line_height(DEFAULT_FONT_SIZE * 1.2);
                    current_layer.write_text(paragraph, &doc);
                }
                TextDirection::LeftToRight => {
                    current_layer.set_text_cursor(
                        Mm(MARGIN_MM),
                        Mm(y_position)
                    );
                    current_layer.set_line_height(DEFAULT_FONT_SIZE * 1.2);
                    current_layer.write_text(paragraph, &doc);
                }
            }
            
            current_layer.end_text_section();
            y_position -= DEFAULT_FONT_SIZE * 1.5;
            
            if y_position < MARGIN_MM {
                let (page, layer) = doc.add_page(
                    Mm(PAGE_WIDTH_MM),
                    Mm(PAGE_HEIGHT_MM),
                    "New Layer"
                );
                y_position = PAGE_HEIGHT_MM - MARGIN_MM;
            }
        }
        
        doc.save(&mut File::create(path)?)?;
        Ok(())
    }

    pub fn read_excel<P: AsRef<Path>>(path: P) -> Result<Vec<ExcelSheet>> {
        let mut workbook: Xlsx<_> = open_workbook(path)?;
        let mut sheets = Vec::new();
        
        for sheet_name in workbook.sheet_names().to_owned() {
            if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
                let mut cells = Vec::new();
                
                for (row_idx, row) in range.rows().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        let value = match cell {
                            DataType::Empty => String::new(),
                            DataType::String(s) => s.to_string(),
                            DataType::Float(f) => f.to_string(),
                            DataType::Int(i) => i.to_string(),
                            DataType::Bool(b) => b.to_string(),
                            DataType::Error(_) => String::from("#ERROR"),
                            DataType::DateTime(d) => d.to_string(),
                        };
                        
                        cells.push(ExcelCell {
                            value,
                            row: row_idx as u32,
                            col: col_idx as u32,
                            is_formula: false, // TODO: לזהות נוסחאות
                        });
                    }
                }
                
                sheets.push(ExcelSheet {
                    name: sheet_name,
                    cells,
                });
            }
        }
        
        Ok(sheets)
    }

    pub fn write_excel(sheets: &[ExcelSheet], path: &Path) -> Result<()> {
        let workbook = Workbook::new(path.to_str().unwrap())?;
        let mut sheet_refs = HashMap::new();
        
        for sheet in sheets {
            let worksheet = workbook.add_worksheet(Some(&sheet.name))?;
            sheet_refs.insert(&sheet.name, worksheet);
            
            for cell in &sheet.cells {
                sheet_refs.get(&sheet.name).unwrap()
                    .write_string(cell.row, cell.col, &cell.value, None)?;
            }
        }
        
        workbook.close()?;
        Ok(())
    }

    pub fn read_docx<P: AsRef<Path>>(path: P) -> Result<String> {
        let file = File::open(path)?;
        let doc = DocxDocument::from_file(file)
            .context("טעינת קובץ DOCX נכשלה")?;
        
        let mut text = String::new();
        for paragraph in doc.paragraphs {
            text.push_str(&paragraph.text);
            text.push('\n');
        }
        
        Ok(text)
    }

    pub fn write_docx(text: &str, path: &Path) -> Result<()> {
        let mut doc = DocxDocument::default();
        
        // פיצול הטקסט לפסקאות
        for paragraph in text.split('\n') {
            if !paragraph.trim().is_empty() {
                doc.add_paragraph(paragraph);
            }
        }
        
        let temp_file = NamedTempFile::new()?;
        doc.write_file(&temp_file)?;
        std::fs::copy(temp_file.path(), path)?;
        
        Ok(())
    }

    pub fn read_txt<P: AsRef<Path>>(path: P) -> Result<String> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        
        // ניסיון לקרוא בקידוד UTF-8
        if let Ok(text) = String::from_utf8(bytes.clone()) {
            return Ok(text);
        }
        
        // ניסיון לקרוא בקידוד Windows-1255
        let (cow, _, had_errors) = WINDOWS_1255.decode(&bytes);
        if !had_errors {
            return Ok(cow.into_owned());
        }
        
        anyhow::bail!("לא ניתן לקרוא את הקובץ בקידוד תקין")
    }

    pub fn write_txt(text: &str, path: &Path) -> Result<()> {
        std::fs::write(path, text)?;
        Ok(())
    }

    pub fn read_csv<P: AsRef<Path>>(path: P) -> Result<Vec<ExcelSheet>> {
        let mut rdr = csv::Reader::from_path(path)?;
        let mut cells = Vec::new();
        
        for (row_idx, result) in rdr.records().enumerate() {
            let record = result?;
            for (col_idx, value) in record.iter().enumerate() {
                cells.push(ExcelCell {
                    value: value.to_string(),
                    row: row_idx as u32,
                    col: col_idx as u32,
                    is_formula: false,
                });
            }
        }
        
        Ok(vec![ExcelSheet {
            name: "Sheet1".to_string(),
            cells,
        }])
    }

    pub fn write_csv(sheet: &ExcelSheet, path: &Path) -> Result<()> {
        let mut wtr = csv::Writer::from_path(path)?;
        
        // מיון התאים לפי שורות ועמודות
        let mut rows: HashMap<u32, Vec<(u32, String)>> = HashMap::new();
        for cell in &sheet.cells {
            rows.entry(cell.row)
                .or_insert_with(Vec::new)
                .push((cell.col, cell.value.clone()));
        }
        
        // כתיבת השורות בסדר הנכון
        for row_idx in 0..=*rows.keys().max().unwrap_or(&0) {
            if let Some(row) = rows.get(&row_idx) {
                let mut sorted_row = row.clone();
                sorted_row.sort_by_key(|(col, _)| *col);
                let values: Vec<String> = sorted_row.into_iter().map(|(_, val)| val).collect();
                wtr.write_record(&values)?;
            }
        }
        
        wtr.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum TextDirection {
    LeftToRight,
    RightToLeft,
} 