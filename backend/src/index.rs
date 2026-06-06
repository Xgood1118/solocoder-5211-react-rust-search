use std::collections::HashMap;
use parking_lot::RwLock;
use rayon::prelude::*;

use crate::models::Document;
use crate::tokenizer::tokenize;

#[derive(Debug, Clone)]
pub struct PostingEntry {
    pub doc_id: String,
    pub positions: Vec<usize>,
    pub tf: usize,
}

pub struct InvertedIndex {
    pub postings: HashMap<String, Vec<PostingEntry>>,
    pub doc_lengths: HashMap<String, usize>,
    pub avgdl: f32,
    pub doc_count: usize,
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex {
            postings: HashMap::new(),
            doc_lengths: HashMap::new(),
            avgdl: 0.0,
            doc_count: 0,
        }
    }

    pub fn build(docs: &[Document]) -> Self {
        let mut index = InvertedIndex::new();

        for doc in docs {
            index.add_document(doc);
        }

        index.compute_avgdl();
        index
    }

    pub fn add_document(&mut self, doc: &Document) {
        let searchable_text = format!("{} {} {}", doc.title, doc.tags.join(" "), doc.content);
        let tokens = tokenize(&searchable_text);
        let doc_len = tokens.len();

        let mut token_positions: HashMap<String, Vec<usize>> = HashMap::new();
        for (pos, token) in tokens.iter().enumerate() {
            token_positions
                .entry(token.clone())
                .or_default()
                .push(pos);
        }

        for (token, positions) in &token_positions {
            let entry = PostingEntry {
                doc_id: doc.id.clone(),
                positions: positions.clone(),
                tf: positions.len(),
            };

            self.postings
                .entry(token.clone())
                .or_default()
                .push(entry);
        }

        self.doc_lengths.insert(doc.id.clone(), doc_len);
        self.doc_count += 1;
    }

    pub fn remove_document(&mut self, doc_id: &str) {
        for posting_list in self.postings.values_mut() {
            posting_list.retain(|entry| entry.doc_id != doc_id);
        }
        self.doc_lengths.remove(doc_id);
        self.doc_count = self.doc_lengths.len();
    }

    pub fn compute_avgdl(&mut self) {
        let total_len: usize = self.doc_lengths.values().sum();
        self.avgdl = if self.doc_count > 0 {
            total_len as f32 / self.doc_count as f32
        } else {
            0.0
        };
    }

    pub fn bm25_score(&self, doc_id: &str, query_terms: &[String]) -> f32 {
        let k1 = 1.5;
        let b = 0.75;

        let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&0) as f32;
        let avgdl = self.avgdl;

        if avgdl == 0.0 {
            return 0.0;
        }

        let mut score = 0.0;

        for term in query_terms {
            if let Some(posting_list) = self.postings.get(term) {
                let df = posting_list.len() as f32;
                let n = self.doc_count as f32;

                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

                if let Some(entry) = posting_list.iter().find(|e| e.doc_id == doc_id) {
                    let tf = entry.tf as f32;
                    let numerator = tf * (k1 + 1.0);
                    let denominator = tf + k1 * (1.0 - b + b * doc_len / avgdl);
                    score += idf * numerator / denominator;
                }
            }
        }

        score
    }

    pub fn search_and_rank(
        &self,
        query_terms: &[String],
        candidate_ids: &[String],
    ) -> Vec<(String, f32)> {
        candidate_ids
            .par_iter()
            .map(|doc_id| {
                let score = self.bm25_score(doc_id, query_terms);
                (doc_id.clone(), score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect()
    }

    pub fn get_docs_for_term(&self, term: &str) -> Vec<String> {
        self.postings
            .get(term)
            .map(|list| list.iter().map(|e| e.doc_id.clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_term_positions(&self, doc_id: &str, term: &str) -> Vec<usize> {
        self.postings
            .get(term)
            .and_then(|list| list.iter().find(|e| e.doc_id == doc_id))
            .map(|entry| entry.positions.clone())
            .unwrap_or_default()
    }
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IndexStore {
    pub index: RwLock<InvertedIndex>,
}

impl IndexStore {
    pub fn new() -> Self {
        IndexStore {
            index: RwLock::new(InvertedIndex::new()),
        }
    }
}

impl Default for IndexStore {
    fn default() -> Self {
        Self::new()
    }
}
