use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;

/// מנהל אינטגרציות
pub struct IntegrationManager {
    /// אינטגרציית CAD
    cad: Arc<CadIntegration>,
    /// אינטגרציית מסמכים
    documents: Arc<DocumentIntegration>,
    /// אינטגרציית Gmail
    gmail: Arc<GmailIntegration>,
    /// מנהל הגדרות
    settings: Arc<Mutex<IntegrationSettings>>,
}

/// אינטגרציית CAD
pub struct CadIntegration {
    /// חיבורי AutoCAD
    autocad_connections: Arc<Mutex<HashMap<String, AutoCadConnection>>>,
    /// חיבורי Revit
    revit_connections: Arc<Mutex<HashMap<String, RevitConnection>>>,
    /// מטמון קבצים
    file_cache: Arc<Mutex<HashMap<String, CadFile>>>,
}

/// אינטגרציית מסמכים
pub struct DocumentIntegration {
    /// מעבדי מסמכים
    processors: Arc<Mutex<HashMap<String, DocumentProcessor>>>,
    /// תבניות מסמכים
    templates: Arc<Mutex<HashMap<String, DocumentTemplate>>>,
}

/// אינטגרציית Gmail
pub struct GmailIntegration {
    /// הגדרות OAuth
    oauth: Arc<Mutex<OAuthConfig>>,
    /// חיבורי Gmail
    connections: Arc<Mutex<HashMap<String, GmailConnection>>>,
}

impl IntegrationManager {
    pub async fn new(settings: IntegrationSettings) -> Result<Self, IntegrationError> {
        Ok(Self {
            cad: Arc::new(CadIntegration::new().await?),
            documents: Arc::new(DocumentIntegration::new().await?),
            gmail: Arc::new(GmailIntegration::new().await?),
            settings: Arc::new(Mutex::new(settings)),
        })
    }

    /// מייצא לקובץ CAD
    pub async fn export_to_cad(&self, data: &CadData, format: CadFormat) -> Result<PathBuf, IntegrationError> {
        self.cad.export(data, format).await
    }

    /// מייבא מקובץ CAD
    pub async fn import_from_cad(&self, path: &PathBuf) -> Result<CadData, IntegrationError> {
        self.cad.import(path).await
    }

    /// מייצא למסמך
    pub async fn export_to_document(&self, data: &DocumentData, format: DocumentFormat) -> Result<PathBuf, IntegrationError> {
        self.documents.export(data, format).await
    }

    /// מייבא ממסמך
    pub async fn import_from_document(&self, path: &PathBuf) -> Result<DocumentData, IntegrationError> {
        self.documents.import(path).await
    }

    /// שולח אימייל
    pub async fn send_email(&self, email: &Email) -> Result<(), IntegrationError> {
        self.gmail.send(email).await
    }
}

impl CadIntegration {
    pub async fn new() -> Result<Self, IntegrationError> {
        Ok(Self {
            autocad_connections: Arc::new(Mutex::new(HashMap::new())),
            revit_connections: Arc::new(Mutex::new(HashMap::new())),
            file_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// מייצא לקובץ CAD
    pub async fn export(&self, data: &CadData, format: CadFormat) -> Result<PathBuf, IntegrationError> {
        match format {
            CadFormat::AutoCAD => self.export_to_autocad(data).await,
            CadFormat::Revit => self.export_to_revit(data).await,
        }
    }

    /// מייבא מקובץ CAD
    pub async fn import(&self, path: &PathBuf) -> Result<CadData, IntegrationError> {
        // זיהוי פורמט
        let format = self.detect_format(path).await?;
        
        match format {
            CadFormat::AutoCAD => self.import_from_autocad(path).await,
            CadFormat::Revit => self.import_from_revit(path).await,
        }
    }

    /// מייצא ל-AutoCAD
    async fn export_to_autocad(&self, data: &CadData) -> Result<PathBuf, IntegrationError> {
        let mut connections = self.autocad_connections.lock().await;
        
        // יצירת חיבור חדש אם צריך
        if connections.is_empty() {
            let connection = AutoCadConnection::new().await?;
            connections.insert("default".to_string(), connection);
        }
        
        let connection = connections.get("default").unwrap();
        connection.export(data).await
    }

    /// מייבא מ-AutoCAD
    async fn import_from_autocad(&self, path: &PathBuf) -> Result<CadData, IntegrationError> {
        let mut connections = self.autocad_connections.lock().await;
        
        // יצירת חיבור חדש אם צריך
        if connections.is_empty() {
            let connection = AutoCadConnection::new().await?;
            connections.insert("default".to_string(), connection);
        }
        
        let connection = connections.get("default").unwrap();
        connection.import(path).await
    }
}

impl DocumentIntegration {
    pub async fn new() -> Result<Self, IntegrationError> {
        Ok(Self {
            processors: Arc::new(Mutex::new(HashMap::new())),
            templates: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// מייצא למסמך
    pub async fn export(&self, data: &DocumentData, format: DocumentFormat) -> Result<PathBuf, IntegrationError> {
        let mut processors = self.processors.lock().await;
        
        // יצירת מעבד חדש אם צריך
        if !processors.contains_key(&format.to_string()) {
            let processor = DocumentProcessor::new(format).await?;
            processors.insert(format.to_string(), processor);
        }
        
        let processor = processors.get(&format.to_string()).unwrap();
        processor.export(data).await
    }

    /// מייבא ממסמך
    pub async fn import(&self, path: &PathBuf) -> Result<DocumentData, IntegrationError> {
        // זיהוי פורמט
        let format = self.detect_format(path).await?;
        
        let mut processors = self.processors.lock().await;
        
        // יצירת מעבד חדש אם צריך
        if !processors.contains_key(&format.to_string()) {
            let processor = DocumentProcessor::new(format).await?;
            processors.insert(format.to_string(), processor);
        }
        
        let processor = processors.get(&format.to_string()).unwrap();
        processor.import(path).await
    }
}

impl GmailIntegration {
    pub async fn new() -> Result<Self, IntegrationError> {
        Ok(Self {
            oauth: Arc::new(Mutex::new(OAuthConfig::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// שולח אימייל
    pub async fn send(&self, email: &Email) -> Result<(), IntegrationError> {
        let mut connections = self.connections.lock().await;
        
        // יצירת חיבור חדש אם צריך
        if connections.is_empty() {
            let oauth = self.oauth.lock().await;
            let connection = GmailConnection::new(&oauth).await?;
            connections.insert("default".to_string(), connection);
        }
        
        let connection = connections.get("default").unwrap();
        connection.send(email).await
    }
} 