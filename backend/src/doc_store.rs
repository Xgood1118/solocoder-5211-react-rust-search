use std::collections::HashMap;
use parking_lot::RwLock;
use chrono::Utc;

use crate::models::{Document, CreateDocumentRequest, UpdateDocumentRequest};
use crate::markdown::markdown_to_html;
use crate::models::new_doc_id;

pub struct DocumentStore {
    docs: HashMap<String, Document>,
}

impl DocumentStore {
    pub fn new() -> Self {
        DocumentStore {
            docs: HashMap::new(),
        }
    }

    pub fn get_all(&self) -> Vec<Document> {
        self.docs.values().cloned().collect()
    }

    pub fn get_by_id(&self, id: &str) -> Option<Document> {
        self.docs.get(id).cloned()
    }

    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<Document> {
        self.docs
            .values()
            .filter(|doc| tags.iter().all(|t| doc.tags.contains(t)))
            .cloned()
            .collect()
    }

    pub fn filter_by_author(&self, author: &str) -> Vec<Document> {
        self.docs
            .values()
            .filter(|doc| doc.author == author)
            .cloned()
            .collect()
    }

    pub fn filter_by_tags_and_author(&self, tags: &[String], author: &str) -> Vec<Document> {
        self.docs
            .values()
            .filter(|doc| {
                doc.author == author && tags.iter().all(|t| doc.tags.contains(t))
            })
            .cloned()
            .collect()
    }

    pub fn insert(&mut self, doc: Document) {
        self.docs.insert(doc.id.clone(), doc);
    }

    pub fn create_from_request(&self, req: CreateDocumentRequest) -> Document {
        let html = markdown_to_html(&req.content);
        Document {
            id: new_doc_id(),
            title: req.title,
            author: req.author,
            tags: req.tags,
            created_at: Utc::now(),
            content: req.content,
            html,
        }
    }

    pub fn update(&mut self, id: &str, req: UpdateDocumentRequest) -> Option<Document> {
        if let Some(existing) = self.docs.get(id).cloned() {
            let mut updated = existing.clone();
            if let Some(title) = req.title {
                updated.title = title;
            }
            if let Some(author) = req.author {
                updated.author = author;
            }
            if let Some(tags) = req.tags {
                updated.tags = tags;
            }
            if let Some(content) = req.content {
                updated.content = content;
                updated.html = markdown_to_html(&updated.content);
            }
            self.docs.insert(id.to_string(), updated.clone());
            Some(updated)
        } else {
            None
        }
    }

    pub fn delete(&mut self, id: &str) -> bool {
        self.docs.remove(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DocumentStoreLock {
    pub store: RwLock<DocumentStore>,
}

impl DocumentStoreLock {
    pub fn new() -> Self {
        DocumentStoreLock {
            store: RwLock::new(DocumentStore::new()),
        }
    }
}

impl Default for DocumentStoreLock {
    fn default() -> Self {
        Self::new()
    }
}
