use crate::doc_store::DocumentStoreLock;
use crate::index::IndexStore;
use crate::models::SearchResult;
use crate::query_parser::{parse_query, extract_search_terms, ParsedQuery};
use crate::tokenizer::tokenize;

pub struct SearchService;

impl SearchService {
    pub fn search(
        doc_store: &DocumentStoreLock,
        index_store: &IndexStore,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> (Vec<SearchResult>, usize, Vec<String>, Vec<String>) {
        let parsed = parse_query(query);
        let search_terms = extract_search_terms(&parsed);
        let display_terms = search_terms.clone();

        if search_terms.is_empty() && parsed.phrase_terms.is_empty()
            && parsed.tag_filters.is_empty() && parsed.author_filter.is_none()
            && parsed.after_date.is_none() && parsed.before_date.is_none()
        {
            return (Vec::new(), 0, Vec::new(), display_terms);
        }

        let docs_read = doc_store.store.read();
        let index_read = index_store.index.read();

        let mut candidate_ids: Vec<String> = if !search_terms.is_empty() || !parsed.phrase_terms.is_empty() {
            let mut all_terms = search_terms.clone();
            for phrase in &parsed.phrase_terms {
                all_terms.extend(phrase.clone());
            }

            let mut doc_sets: Vec<std::collections::HashSet<String>> = Vec::new();

            for term in &search_terms {
                let docs = index_read.get_docs_for_term(term);
                doc_sets.push(docs.into_iter().collect());
            }

            for phrase in &parsed.phrase_terms {
                if let Some(first_term) = phrase.first() {
                    let docs = index_read.get_docs_for_term(first_term);
                    doc_sets.push(docs.into_iter().collect());
                }
            }

            if doc_sets.is_empty() {
                Vec::new()
            } else {
                let mut result = doc_sets[0].clone();
                for set in doc_sets.iter().skip(1) {
                    result = result.intersection(set).cloned().collect();
                }
                result.into_iter().collect()
            }
        } else {
            docs_read.get_all().iter().map(|d| d.id.clone()).collect()
        };

        if !parsed.tag_filters.is_empty() {
            let tag_filtered: std::collections::HashSet<String> = docs_read
                .filter_by_tags(&parsed.tag_filters)
                .iter()
                .map(|d| d.id.clone())
                .collect();
            candidate_ids = candidate_ids
                .into_iter()
                .filter(|id| tag_filtered.contains(id))
                .collect();
        }

        if let Some(author) = &parsed.author_filter {
            let author_filtered: std::collections::HashSet<String> = docs_read
                .filter_by_author(author)
                .iter()
                .map(|d| d.id.clone())
                .collect();
            candidate_ids = candidate_ids
                .into_iter()
                .filter(|id| author_filtered.contains(id))
                .collect();
        }

        if let Some(after) = parsed.after_date {
            candidate_ids.retain(|id| {
                docs_read.get_by_id(id)
                    .map(|d| d.created_at >= after)
                    .unwrap_or(false)
            });
        }

        if let Some(before) = parsed.before_date {
            candidate_ids.retain(|id| {
                docs_read.get_by_id(id)
                    .map(|d| d.created_at <= before)
                    .unwrap_or(false)
            });
        }

        let all_terms = extract_search_terms(&parsed);

        let mut scored: Vec<(String, f32)> = if !all_terms.is_empty() {
            index_read.search_and_rank(&all_terms, &candidate_ids)
        } else {
            candidate_ids.into_iter().map(|id| (id, 0.0)).collect()
        };

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let total = scored.len();

        let paginated = scored.into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();

        let results: Vec<SearchResult> = paginated
            .into_iter()
            .filter_map(|(doc_id, score)| {
                docs_read.get_by_id(&doc_id).map(|doc| {
                    let snippet = Self::generate_snippet(
                        &doc.content,
                        &all_terms,
                        &parsed,
                    );
                    SearchResult {
                        document: doc,
                        score,
                        snippet,
                    }
                })
            })
            .collect();

        let all_tokens = Self::get_all_tokens(&parsed);

        (results, total, all_tokens, display_terms)
    }

    fn get_all_tokens(parsed: &ParsedQuery) -> Vec<String> {
        let mut tokens = Vec::new();
        for term in &parsed.search_terms {
            tokens.push(term.clone());
        }
        for phrase in &parsed.phrase_terms {
            tokens.extend(phrase.clone());
        }
        for tag in &parsed.tag_filters {
            tokens.push(format!("tag:{}", tag));
        }
        if let Some(author) = &parsed.author_filter {
            tokens.push(format!("author:{}", author));
        }
        if let Some(after) = &parsed.after_date {
            tokens.push(format!("after:{}", after.format("%Y-%m-%d")));
        }
        if let Some(before) = &parsed.before_date {
            tokens.push(format!("before:{}", before.format("%Y-%m-%d")));
        }
        tokens
    }

    fn generate_snippet(content: &str, terms: &[String], _parsed: &ParsedQuery) -> String {
        let snippet_window = 30;
        let chars: Vec<char> = content.chars().collect();

        let term_chars_list: Vec<Vec<char>> = terms
            .iter()
            .map(|t| t.chars().flat_map(|c| c.to_lowercase()).collect())
            .filter(|v: &Vec<char>| !v.is_empty())
            .collect();

        if term_chars_list.is_empty() {
            let snippet: String = chars.iter().take(snippet_window * 2).collect();
            return snippet;
        }

        let content_lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();

        let mut char_positions: Vec<(usize, usize)> = Vec::new();

        for term_chars in &term_chars_list {
            let term_len = term_chars.len();
            let mut i = 0;
            while i + term_len <= content_lower.len() {
                if &content_lower[i..i + term_len] == term_chars.as_slice() {
                    char_positions.push((i, term_len));
                    i += 1;
                } else {
                    i += 1;
                }
            }
        }

        if char_positions.is_empty() {
            let snippet: String = chars.iter().take(snippet_window * 2).collect();
            return snippet;
        }

        char_positions.sort_by_key(|(pos, _)| *pos);

        let (center, term_len) = char_positions[0];
        let start = if center >= snippet_window {
            center - snippet_window
        } else {
            0
        };
        let end = std::cmp::min(chars.len(), center + term_len + snippet_window);

        let snippet_chars: Vec<char> = chars[start..end].to_vec();
        let mut snippet: String = snippet_chars.into_iter().collect();

        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < chars.len() {
            snippet = format!("{}...", snippet);
        }

        snippet
    }
}
