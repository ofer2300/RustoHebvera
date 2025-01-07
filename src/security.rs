use std::sync::Arc;
use tokio::sync::Mutex;
use ring::aead::{self, BoundKey, Aad, UnboundKey, SealingKey, OpeningKey};

/// מנהל האבטחה המרכזי
pub struct SecurityManager {
    /// מנהל הצפנה
    encryption_manager: Arc<EncryptionManager>,
    /// מנהל אימות
    auth_manager: Arc<AuthenticationManager>,
    /// מנהל התאוששות
    recovery_manager: Arc<RecoveryManager>,
    /// מערכת ניטור
    monitoring: Arc<SecurityMonitoring>,
}

/// מנהל הצפנה
pub struct EncryptionManager {
    /// מפתח הצפנה ראשי
    master_key: Arc<Mutex<UnboundKey>>,
    /// מפתחות הצפנה פעילים
    active_keys: Arc<Mutex<Vec<SealingKey>>>,
}

/// מנהל אימות
pub struct AuthenticationManager {
    /// מדיניות סיסמאות
    password_policy: PasswordPolicy,
    /// מנהל 2FA
    two_factor: TwoFactorAuth,
}

/// מנהל התאוששות
pub struct RecoveryManager {
    /// נקודות שחזור
    recovery_points: Arc<Mutex<Vec<RecoveryPoint>>>,
    /// מדיניות גיבוי
    backup_policy: BackupPolicy,
}

impl SecurityManager {
    pub async fn new() -> Result<Self, SecurityError> {
        Ok(Self {
            encryption_manager: Arc::new(EncryptionManager::new().await?),
            auth_manager: Arc::new(AuthenticationManager::new()),
            recovery_manager: Arc::new(RecoveryManager::new()),
            monitoring: Arc::new(SecurityMonitoring::new()),
        })
    }

    /// מצפין מידע
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let key = self.encryption_manager.get_active_key().await?;
        let aad = Aad::empty(); // Additional Authenticated Data
        
        let mut in_out = data.to_vec();
        key.seal_in_place_append_tag(aad, &mut in_out)
            .map_err(|_| SecurityError::EncryptionFailed)?;
            
        Ok(in_out)
    }

    /// מפענח מידע
    pub async fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let key = self.encryption_manager.get_active_key().await?;
        let aad = Aad::empty();
        
        let mut in_out = encrypted.to_vec();
        key.open_in_place(aad, &mut in_out)
            .map_err(|_| SecurityError::DecryptionFailed)?;
            
        Ok(in_out)
    }

    /// מאמת משתמש
    pub async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken, SecurityError> {
        // בדיקת סיסמה
        self.auth_manager.verify_password(&credentials.password)?;
        
        // בדיקת 2FA
        self.auth_manager.verify_2fa(&credentials.two_factor_code)?;
        
        // יצירת טוקן
        let token = AuthToken::new();
        
        Ok(token)
    }

    /// יוצר נקודת שחזור
    pub async fn create_recovery_point(&self) -> Result<RecoveryPoint, SecurityError> {
        self.recovery_manager.create_point().await
    }

    /// משחזר למצב קודם
    pub async fn restore(&self, point: &RecoveryPoint) -> Result<(), SecurityError> {
        self.recovery_manager.restore(point).await
    }
}

impl EncryptionManager {
    /// יוצר מנהל הצפנה חדש
    pub async fn new() -> Result<Self, SecurityError> {
        let rng = ring::rand::SystemRandom::new();
        let key_bytes: [u8; 32] = ring::rand::generate(&rng)?.expose();
        
        let master_key = UnboundKey::new(&aead::CHACHA20_POLY1305, &key_bytes)
            .map_err(|_| SecurityError::KeyGenerationFailed)?;
            
        Ok(Self {
            master_key: Arc::new(Mutex::new(master_key)),
            active_keys: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// מחזיר מפתח הצפנה פעיל
    pub async fn get_active_key(&self) -> Result<SealingKey, SecurityError> {
        let mut active_keys = self.active_keys.lock().await;
        
        if active_keys.is_empty() {
            let master_key = self.master_key.lock().await;
            let new_key = self.generate_key(&master_key).await?;
            active_keys.push(new_key.clone());
            Ok(new_key)
        } else {
            Ok(active_keys[0].clone())
        }
    }
} 