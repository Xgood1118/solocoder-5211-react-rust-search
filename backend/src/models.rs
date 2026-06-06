use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub total: usize,
    pub results: Vec<SearchResult>,
    pub tokens: Vec<String>,
    pub took_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutocompleteResult {
    pub query: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchImportResult {
    pub success_count: usize,
    pub fail_count: usize,
    pub failures: Vec<BatchFailure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchFailure {
    pub index: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub error: String,
    pub field: Option<String>,
}

pub fn new_doc_id() -> String {
    Uuid::new_v4().to_string()
}
