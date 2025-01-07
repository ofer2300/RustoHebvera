use crate::translation::Translator;
use crate::file_processor::{FileProcessor, FileType, ExcelSheet};
use std::path::Path;
use anyhow::{Result, Context};

pub struct DocumentProcessor {
    translator: Translator,
}

#[derive(Debug)]
pub enum DocumentType {
    Technical,
    Standard,
    Drawing,
    Calculation,
}

impl DocumentProcessor {
    pub fn new(translator: Translator) -> Self {
        Self { translator }
    }

    pub async fn process_document<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        doc_type: DocumentType,
    ) -> Result<()> {
        // זיהוי סוג הקובץ
        let file_type = FileProcessor::detect_file_type(&input_path);
        
        match file_type {
            FileType::PDF => self.process_pdf(&input_path, &output_path).await?,
            FileType::DOCX => self.process_docx(&input_path, &output_path).await?,
            FileType::TXT => self.process_txt(&input_path, &output_path).await?,
            FileType::XLSX => self.process_excel(&input_path, &output_path).await?,
            FileType::CSV => self.process_csv(&input_path, &output_path).await?,
            FileType::Unknown => anyhow::bail!("סוג קובץ לא נתמך"),
        }
        
        Ok(())
    }

    async fn process_pdf<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let content = FileProcessor::read_pdf(&input_path)?;
        let translated_content = self.translator
            .translate(&content)
            .map_err(|e| anyhow::anyhow!("שגיאת תרגום: {:?}", e))?;
        FileProcessor::write_pdf(&translated_content, output_path.as_ref())?;
        Ok(())
    }

    async fn process_docx<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let content = FileProcessor::read_docx(&input_path)?;
        let translated_content = self.translator
            .translate(&content)
            .map_err(|e| anyhow::anyhow!("שגיאת תרגום: {:?}", e))?;
        FileProcessor::write_docx(&translated_content, output_path.as_ref())?;
        Ok(())
    }

    async fn process_txt<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let content = FileProcessor::read_txt(&input_path)?;
        let translated_content = self.translator
            .translate(&content)
            .map_err(|e| anyhow::anyhow!("שגיאת תרגום: {:?}", e))?;
        FileProcessor::write_txt(&translated_content, output_path.as_ref())?;
        Ok(())
    }

    async fn process_excel<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let sheets = FileProcessor::read_excel(&input_path)?;
        let mut translated_sheets = Vec::new();
        
        for sheet in sheets {
            let mut translated_cells = Vec::new();
            
            for cell in sheet.cells {
                let translated_value = if !cell.value.trim().is_empty() {
                    self.translator
                        .translate(&cell.value)
                        .unwrap_or_else(|_| cell.value.clone())
                } else {
                    cell.value.clone()
                };
                
                translated_cells.push(ExcelSheet {
                    name: sheet.name.clone(),
                    cells: vec![ExcelCell {
                        value: translated_value,
                        row: cell.row,
                        col: cell.col,
                        is_formula: cell.is_formula,
                    }],
                });
            }
            
            translated_sheets.push(ExcelSheet {
                name: sheet.name,
                cells: translated_cells.into_iter().flat_map(|s| s.cells).collect(),
            });
        }
        
        FileProcessor::write_excel(&translated_sheets, output_path.as_ref())?;
        Ok(())
    }

    async fn process_csv<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let sheets = FileProcessor::read_csv(&input_path)?;
        let mut translated_sheets = Vec::new();
        
        for sheet in sheets {
            let mut translated_cells = Vec::new();
            
            for cell in sheet.cells {
                let translated_value = if !cell.value.trim().is_empty() {
                    self.translator
                        .translate(&cell.value)
                        .unwrap_or_else(|_| cell.value.clone())
                } else {
                    cell.value.clone()
                };
                
                translated_cells.push(ExcelCell {
                    value: translated_value,
                    row: cell.row,
                    col: cell.col,
                    is_formula: cell.is_formula,
                });
            }
            
            translated_sheets.push(ExcelSheet {
                name: sheet.name,
                cells: translated_cells,
            });
        }
        
        FileProcessor::write_csv(&translated_sheets[0], output_path.as_ref())?;
        Ok(())
    }

    async fn process_technical_doc<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<()> {
        self.process_document(input_path, output_path, DocumentType::Technical).await
    }

    async fn process_standard_doc<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<()> {
        self.process_document(input_path, output_path, DocumentType::Standard).await
    }

    async fn process_drawing_doc<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<()> {
        // TODO: להוסיף תמיכה בעיבוד שרטוטים
        Ok(())
    }

    async fn process_calculation_doc<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<()> {
        self.process_document(input_path, output_path, DocumentType::Calculation).await
    }
} 