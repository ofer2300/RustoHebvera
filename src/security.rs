use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use ring::aead::{self, BoundKey, Aad, UnboundKey, AES_256_GCM};
use ring::rand::SystemRandom;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub two_factor_auth: bool,
    pub session_timeout: chrono::Duration,
    pub password_policy: PasswordPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special: bool,
    pub max_age_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub user_id: String,
    pub password_hash: String,
    pub salt: String,
    pub two_factor_secret: Option<String>,
    pub last_password_change: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditLog {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub action: SecurityAction,
    pub status: SecurityStatus,
    pub details: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    Login,
    Logout,
    PasswordChange,
    TwoFactorSetup,
    AccessAttempt,
    DataEncryption,
    DataDecryption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityStatus {
    Success,
    Failure,
    Warning,
}

pub struct SecurityManager {
    config: Arc<Mutex<SecurityConfig>>,
    credentials: Arc<Mutex<HashMap<String, UserCredentials>>>,
    audit_log: Arc<Mutex<Vec<SecurityAuditLog>>>,
    encryption_key: Arc<Mutex<Option<UnboundKey>>>,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Result<Self> {
        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            credentials: Arc::new(Mutex::new(HashMap::new())),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            encryption_key: Arc::new(Mutex::new(None)),
        })
    }

    // הצפנה מקצה לקצה
    pub async fn initialize_encryption(&self) -> Result<()> {
        let rng = SystemRandom::new();
        let key_bytes: [u8; 32] = ring::rand::generate(&rng)?.expose();
        let key = UnboundKey::new(&AES_256_GCM, &key_bytes)?;
        
        let mut encryption_key = self.encryption_key.lock().await;
        *encryption_key = Some(key);
        
        self.log_security_event(
            "system",
            SecurityAction::DataEncryption,
            SecurityStatus::Success,
            "Encryption initialized",
            None,
        ).await?;
        
        Ok(())
    }

    pub async fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encryption_key = self.encryption_key.lock().await;
        if let Some(key) = &*encryption_key {
            let nonce = ring::aead::Nonce::assume_unique_for_key([0u8; 12]);
            let mut key = aead::LessSafeKey::new(key.clone());
            
            let mut in_out = data.to_vec();
            key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)?;
            
            Ok(in_out)
        } else {
            anyhow::bail!("Encryption not initialized")
        }
    }

    pub async fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let encryption_key = self.encryption_key.lock().await;
        if let Some(key) = &*encryption_key {
            let nonce = ring::aead::Nonce::assume_unique_for_key([0u8; 12]);
            let mut key = aead::LessSafeKey::new(key.clone());
            
            let mut in_out = encrypted_data.to_vec();
            let decrypted_data = key.open_in_place(nonce, Aad::empty(), &mut in_out)?;
            
            Ok(decrypted_data.to_vec())
        } else {
            anyhow::bail!("Encryption not initialized")
        }
    }

    // אימות דו-שלבי
    pub async fn setup_two_factor(&self, user_id: &str) -> Result<String> {
        let secret = base32::encode(
            base32::Alphabet::RFC4648 { padding: true },
            &ring::rand::generate::<[u8; 20]>(&SystemRandom::new())?.expose(),
        );
        
        let mut credentials = self.credentials.lock().await;
        if let Some(creds) = credentials.get_mut(user_id) {
            creds.two_factor_secret = Some(secret.clone());
            
            self.log_security_event(
                user_id,
                SecurityAction::TwoFactorSetup,
                SecurityStatus::Success,
                "2FA enabled",
                None,
            ).await?;
            
            Ok(secret)
        } else {
            anyhow::bail!("User not found")
        }
    }

    pub async fn verify_two_factor(&self, user_id: &str, code: &str) -> Result<bool> {
        let credentials = self.credentials.lock().await;
        if let Some(creds) = credentials.get(user_id) {
            if let Some(secret) = &creds.two_factor_secret {
                // אימות קוד
                let totp = totp_rs::TOTP::new(
                    totp_rs::Algorithm::SHA1,
                    6,
                    1,
                    30,
                    secret.as_bytes(),
                )?;
                
                let valid = totp.check_current(code)?;
                
                self.log_security_event(
                    user_id,
                    SecurityAction::AccessAttempt,
                    if valid {
                        SecurityStatus::Success
                    } else {
                        SecurityStatus::Failure
                    },
                    "2FA verification",
                    None,
                ).await?;
                
                Ok(valid)
            } else {
                anyhow::bail!("2FA not setup for user")
            }
        } else {
            anyhow::bail!("User not found")
        }
    }

    // ניטור אבטחה
    pub async fn log_security_event(
        &self,
        user_id: &str,
        action: SecurityAction,
        status: SecurityStatus,
        details: &str,
        ip_address: Option<String>,
    ) -> Result<()> {
        let event = SecurityAuditLog {
            timestamp: chrono::Utc::now(),
            user_id: user_id.to_string(),
            action,
            status,
            details: details.to_string(),
            ip_address,
        };
        
        let mut audit_log = self.audit_log.lock().await;
        audit_log.push(event);
        
        Ok(())
    }

    pub async fn get_security_events(&self, user_id: &str) -> Result<Vec<SecurityAuditLog>> {
        let audit_log = self.audit_log.lock().await;
        Ok(audit_log
            .iter()
            .filter(|event| event.user_id == user_id)
            .cloned()
            .collect())
    }

    // מדיניות הרשאות
    pub async fn validate_password(&self, password: &str) -> Result<bool> {
        let config = self.config.lock().await;
        let policy = &config.password_policy;
        
        if password.len() < policy.min_length {
            return Ok(false);
        }
        
        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Ok(false);
        }
        
        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Ok(false);
        }
        
        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Ok(false);
        }
        
        if policy.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Ok(false);
        }
        
        Ok(true)
    }

    pub async fn check_password_expiry(&self, user_id: &str) -> Result<bool> {
        let config = self.config.lock().await;
        let credentials = self.credentials.lock().await;
        
        if let Some(creds) = credentials.get(user_id) {
            let age = chrono::Utc::now() - creds.last_password_change;
            Ok(age.num_days() > config.password_policy.max_age_days as i64)
        } else {
            anyhow::bail!("User not found")
        }
    }
} 