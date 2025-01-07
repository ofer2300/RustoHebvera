use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use crate::technical_dictionary::{TechnicalTerm, TechnicalDictionary};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryVersion {
    pub version_id: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: String,
    pub terms: HashMap<String, TechnicalTerm>,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryMergeReport {
    pub added_terms: Vec<TechnicalTerm>,
    pub updated_terms: Vec<TechnicalTerm>,
    pub conflicting_terms: Vec<(TechnicalTerm, TechnicalTerm)>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermChangeHistory {
    pub term_id: String,
    pub changes: Vec<TermChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermChange {
    pub timestamp: DateTime<Utc>,
    pub changed_by: String,
    pub change_type: ChangeType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Addition,
    Modification,
    Deletion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    pub user_id: String,
    pub name: String,
    pub role: CollaboratorRole,
    pub last_active: DateTime<Utc>,
    pub current_activity: Option<CollaboratorActivity>,
    pub edit_history: VecDeque<TermChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaboratorRole {
    Admin,
    Editor,
    Reviewer,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorActivity {
    pub activity_type: ActivityType,
    pub term_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub status: ActivityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Editing,
    Reviewing,
    Comparing,
    Exporting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityStatus {
    InProgress,
    Pending,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditLock {
    pub term_id: String,
    pub locked_by: String,
    pub locked_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub request_id: String,
    pub term_id: String,
    pub requested_by: String,
    pub requested_at: DateTime<Utc>,
    pub reviewers: Vec<String>,
    pub status: ReviewStatus,
    pub comments: Vec<ReviewComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    InReview,
    Approved,
    Rejected,
    NeedsChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub term_id: String,
    pub resolved_by: String,
    pub timestamp: DateTime<Utc>,
    pub resolution_type: ResolutionType,
    pub comments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionType {
    KeepBase,
    AcceptChanges,
    Merge,
    Custom,
}

pub struct KnowledgeManager {
    versions: HashMap<String, DictionaryVersion>,
    change_history: HashMap<String, TermChangeHistory>,
    collaborators: HashMap<String, CollaboratorInfo>,
    edit_locks: HashMap<String, EditLock>,
    review_requests: HashMap<String, ReviewRequest>,
    conflict_resolutions: Vec<ConflictResolution>,
    last_sync: DateTime<Utc>,
    activity_log: VecDeque<CollaboratorActivity>,
}

impl KnowledgeManager {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            change_history: HashMap::new(),
            collaborators: HashMap::new(),
            edit_locks: HashMap::new(),
            review_requests: HashMap::new(),
            conflict_resolutions: Vec::new(),
            last_sync: Utc::now(),
            activity_log: VecDeque::with_capacity(1000),
        }
    }

    pub fn create_version(
        &mut self,
        dictionary: &TechnicalDictionary,
        version_id: String,
        created_by: String,
        description: String,
    ) -> Result<()> {
        let version = DictionaryVersion {
            version_id: version_id.clone(),
            created_at: Utc::now(),
            created_by,
            description,
            terms: dictionary.get_all_terms_map()?,
            tags: dictionary.get_all_tags_set()?,
        };

        self.versions.insert(version_id, version);
        Ok(())
    }

    pub fn merge_dictionaries(
        &self,
        base_dict: &mut TechnicalDictionary,
        other_dict: &TechnicalDictionary,
    ) -> Result<DictionaryMergeReport> {
        let mut report = DictionaryMergeReport {
            added_terms: Vec::new(),
            updated_terms: Vec::new(),
            conflicting_terms: Vec::new(),
            timestamp: Utc::now(),
        };

        for (term_id, other_term) in other_dict.get_all_terms_map()? {
            match base_dict.get_term(&term_id) {
                Some(base_term) => {
                    if base_term.last_updated < other_term.last_updated {
                        // העדכון בדיקשנרי השני חדש יותר
                        report.updated_terms.push(other_term.clone());
                        base_dict.update_term(&term_id, other_term.clone())?;
                    } else if base_term.last_updated > other_term.last_updated {
                        // אין צורך לעדכן
                        continue;
                    } else {
                        // קונפליקט - אותו זמן עדכון
                        report.conflicting_terms.push((base_term.clone(), other_term.clone()));
                    }
                }
                None => {
                    // מונח חדש
                    report.added_terms.push(other_term.clone());
                    base_dict.add_term(other_term.clone())?;
                }
            }
        }

        Ok(report)
    }

    pub fn track_change(
        &mut self,
        term_id: String,
        changed_by: String,
        change_type: ChangeType,
        field: String,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let change = TermChange {
            timestamp: Utc::now(),
            changed_by,
            change_type,
            old_value,
            new_value,
            field,
        };

        self.change_history
            .entry(term_id.clone())
            .or_insert_with(|| TermChangeHistory {
                term_id,
                changes: Vec::new(),
            })
            .changes
            .push(change);
    }

    pub fn get_term_history(&self, term_id: &str) -> Option<&TermChangeHistory> {
        self.change_history.get(term_id)
    }

    pub fn get_version(&self, version_id: &str) -> Option<&DictionaryVersion> {
        self.versions.get(version_id)
    }

    pub fn get_all_versions(&self) -> Vec<&DictionaryVersion> {
        let mut versions: Vec<_> = self.versions.values().collect();
        versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        versions
    }

    pub fn compare_versions(
        &self,
        version1_id: &str,
        version2_id: &str,
    ) -> Option<DictionaryMergeReport> {
        let v1 = self.versions.get(version1_id)?;
        let v2 = self.versions.get(version2_id)?;

        let mut report = DictionaryMergeReport {
            added_terms: Vec::new(),
            updated_terms: Vec::new(),
            conflicting_terms: Vec::new(),
            timestamp: Utc::now(),
        };

        // מציאת מונחים שנוספו או עודכנו
        for (term_id, term_v2) in &v2.terms {
            match v1.terms.get(term_id) {
                Some(term_v1) => {
                    if term_v1 != term_v2 {
                        report.updated_terms.push(term_v2.clone());
                    }
                }
                None => {
                    report.added_terms.push(term_v2.clone());
                }
            }
        }

        // מציאת קונפליקטים
        for (term_id, term_v1) in &v1.terms {
            if let Some(term_v2) = v2.terms.get(term_id) {
                if term_v1.last_updated == term_v2.last_updated && term_v1 != term_v2 {
                    report.conflicting_terms.push((term_v1.clone(), term_v2.clone()));
                }
            }
        }

        Some(report)
    }

    pub fn add_collaborator(&mut self, user_id: String, name: String, role: CollaboratorRole) {
        let collaborator = CollaboratorInfo {
            user_id: user_id.clone(),
            name,
            role,
            last_active: Utc::now(),
            current_activity: None,
            edit_history: VecDeque::with_capacity(100),
        };
        self.collaborators.insert(user_id, collaborator);
    }

    pub fn update_collaborator_activity(&mut self, user_id: &str, activity: CollaboratorActivity) {
        if let Some(collaborator) = self.collaborators.get_mut(user_id) {
            collaborator.last_active = Utc::now();
            collaborator.current_activity = Some(activity.clone());
            self.activity_log.push_front(activity);
            if self.activity_log.len() > 1000 {
                self.activity_log.pop_back();
            }
        }
    }

    pub fn acquire_edit_lock(&mut self, term_id: String, user_id: String) -> Result<bool> {
        // בדיקה אם המונח כבר נעול
        if let Some(lock) = self.edit_locks.get(&term_id) {
            if lock.expires_at > Utc::now() {
                return Ok(false);
            }
        }

        // יצירת נעילה חדשה
        let lock = EditLock {
            term_id: term_id.clone(),
            locked_by: user_id,
            locked_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(30),
        };
        self.edit_locks.insert(term_id, lock);
        Ok(true)
    }

    pub fn release_edit_lock(&mut self, term_id: &str, user_id: &str) -> Result<bool> {
        if let Some(lock) = self.edit_locks.get(term_id) {
            if lock.locked_by == user_id {
                self.edit_locks.remove(term_id);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn create_review_request(
        &mut self,
        term_id: String,
        requested_by: String,
        reviewers: Vec<String>,
    ) -> Result<String> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let request = ReviewRequest {
            request_id: request_id.clone(),
            term_id,
            requested_by,
            requested_at: Utc::now(),
            reviewers,
            status: ReviewStatus::Pending,
            comments: Vec::new(),
        };
        self.review_requests.insert(request_id.clone(), request);
        Ok(request_id)
    }

    pub fn add_review_comment(
        &mut self,
        request_id: &str,
        author: String,
        content: String,
        field: Option<String>,
    ) -> Result<()> {
        if let Some(request) = self.review_requests.get_mut(request_id) {
            let comment = ReviewComment {
                author,
                timestamp: Utc::now(),
                content,
                field,
            };
            request.comments.push(comment);
        }
        Ok(())
    }

    pub fn update_review_status(&mut self, request_id: &str, status: ReviewStatus) -> Result<()> {
        if let Some(request) = self.review_requests.get_mut(request_id) {
            request.status = status;
        }
        Ok(())
    }

    pub fn resolve_conflict(
        &mut self,
        term_id: String,
        resolved_by: String,
        resolution_type: ResolutionType,
        comments: String,
    ) -> Result<()> {
        let resolution = ConflictResolution {
            term_id,
            resolved_by,
            timestamp: Utc::now(),
            resolution_type,
            comments,
        };
        self.conflict_resolutions.push(resolution);
        Ok(())
    }

    pub fn get_active_collaborators(&self) -> Vec<&CollaboratorInfo> {
        let threshold = Utc::now() - Duration::minutes(15);
        self.collaborators
            .values()
            .filter(|c| c.last_active > threshold)
            .collect()
    }

    pub fn get_collaborator_activity_log(&self, user_id: &str) -> Vec<&CollaboratorActivity> {
        self.activity_log
            .iter()
            .filter(|a| {
                if let Some(term_id) = &a.term_id {
                    if let Some(collaborator) = self.collaborators.get(user_id) {
                        collaborator.edit_history.iter().any(|c| c.term_id == term_id)
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_pending_reviews(&self, reviewer_id: &str) -> Vec<&ReviewRequest> {
        self.review_requests
            .values()
            .filter(|r| {
                r.reviewers.contains(&reviewer_id.to_string()) && 
                matches!(r.status, ReviewStatus::Pending | ReviewStatus::InReview)
            })
            .collect()
    }

    pub fn get_edit_conflicts(&self) -> Vec<(&String, &EditLock)> {
        let now = Utc::now();
        self.edit_locks
            .iter()
            .filter(|(_, lock)| {
                // בדיקה אם יש יותר ממשתמש אחד שמנסה לערוך את אותו מונח
                self.activity_log
                    .iter()
                    .filter(|a| {
                        matches!(a.activity_type, ActivityType::Editing) &&
                        a.term_id.as_ref() == Some(lock.term_id.as_str()) &&
                        a.started_at > now - Duration::minutes(30)
                    })
                    .count() > 1
            })
            .collect()
    }

    pub fn update_last_sync(&mut self) {
        self.last_sync = Utc::now();
    }

    pub fn get_last_sync(&self) -> DateTime<Utc> {
        self.last_sync
    }
} 