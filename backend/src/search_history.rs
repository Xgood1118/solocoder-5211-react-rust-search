use std::collections::{HashMap, VecDeque};
use parking_lot::RwLock;
use chrono::{DateTime, Utc};

use crate::models::SearchHistoryEntry;

pub struct SearchHistory {
    entries: VecDeque<SearchHistoryEntry>,
    query_counts: HashMap<String, u64>,
    max_entries: usize,
}

impl SearchHistory {
    pub fn new() -> Self {
        SearchHistory {
            entries: VecDeque::new(),
            query_counts: HashMap::new(),
            max_entries: 10000,
        }
    }

    pub fn add(&mut self, query: &str, ip: Option<String>) {
        let entry = SearchHistoryEntry {
            query: query.to_string(),
            timestamp: Utc::now(),
            ip,
        };

        *self.query_counts.entry(query.to_string()).or_insert(0) += 1;

        self.entries.push_front(entry);
        if self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
    }

    pub fn autocomplete(&self, prefix: &str, limit: usize) -> Vec<(String, u64)> {
        let mut results: Vec<(String, u64)> = self.query_counts
            .iter()
            .filter(|(q, _)| q.contains(prefix) && !q.is_empty())
            .map(|(q, c)| (q.clone(), *c))
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        results.truncate(limit);
        results
    }

    pub fn list(&self, limit: usize) -> Vec<SearchHistoryEntry> {
        self.entries.iter().take(limit).cloned().collect()
    }

    pub fn delete(&mut self, query: &str) {
        self.entries.retain(|e| e.query != query);
        self.query_counts.remove(query);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.query_counts.clear();
    }
}

impl Default for SearchHistory {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SearchHistoryStore {
    pub history: RwLock<SearchHistory>,
}

impl SearchHistoryStore {
    pub fn new() -> Self {
        SearchHistoryStore {
            history: RwLock::new(SearchHistory::new()),
        }
    }
}

impl Default for SearchHistoryStore {
    fn default() -> Self {
        Self::new()
    }
}
