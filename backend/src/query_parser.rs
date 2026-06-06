use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone)]
pub struct ParsedQuery {
    pub search_terms: Vec<String>,
    pub phrase_terms: Vec<Vec<String>>,
    pub tag_filters: Vec<String>,
    pub author_filter: Option<String>,
    pub after_date: Option<DateTime<Utc>>,
    pub before_date: Option<DateTime<Utc>>,
}

pub fn parse_query(query: &str) -> ParsedQuery {
    let mut result = ParsedQuery {
        search_terms: Vec::new(),
        phrase_terms: Vec::new(),
        tag_filters: Vec::new(),
        author_filter: None,
        after_date: None,
        before_date: None,
    };

    let tokens = tokenize_query(query);

    for token in tokens {
        if let Some(stripped) = token.strip_prefix("tag:") {
            if !stripped.is_empty() {
                result.tag_filters.push(stripped.to_string());
            }
        } else if let Some(stripped) = token.strip_prefix("author:") {
            if !stripped.is_empty() {
                result.author_filter = Some(stripped.to_string());
            }
        } else if let Some(stripped) = token.strip_prefix("after:") {
            if let Ok(date) = NaiveDate::parse_from_str(stripped, "%Y-%m-%d") {
                let dt = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                result.after_date = Some(dt);
            }
        } else if let Some(stripped) = token.strip_prefix("before:") {
            if let Ok(date) = NaiveDate::parse_from_str(stripped, "%Y-%m-%d") {
                let dt = date.and_hms_opt(23, 59, 59).unwrap().and_utc();
                result.before_date = Some(dt);
            }
        } else if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
            let phrase = &token[1..token.len() - 1];
            let terms = crate::tokenizer::tokenize(phrase);
            if !terms.is_empty() {
                result.phrase_terms.push(terms);
            }
        } else {
            let terms = crate::tokenizer::tokenize(&token);
            result.search_terms.extend(terms);
        }
    }

    result
}

fn tokenize_query(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = query.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '"' {
            let mut phrase = String::from('"');
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                phrase.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                phrase.push('"');
                i += 1;
            }
            if phrase.len() > 2 {
                tokens.push(phrase);
            }
        } else if c.is_whitespace() {
            i += 1;
        } else {
            let mut word = String::new();
            while i < chars.len() && !chars[i].is_whitespace() {
                word.push(chars[i]);
                i += 1;
            }
            if !word.is_empty() {
                tokens.push(word);
            }
        }
    }

    tokens
}

pub fn extract_search_terms(parsed: &ParsedQuery) -> Vec<String> {
    let mut all_terms = parsed.search_terms.clone();
    for phrase in &parsed.phrase_terms {
        all_terms.extend(phrase.clone());
    }
    all_terms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let parsed = parse_query("rust memory");
        assert_eq!(parsed.search_terms, vec!["rust", "memory"]);
        assert!(parsed.tag_filters.is_empty());
        assert!(parsed.author_filter.is_none());
    }

    #[test]
    fn test_tag_filter() {
        let parsed = parse_query("rust tag:backend");
        assert!(parsed.search_terms.contains(&"rust".to_string()));
        assert_eq!(parsed.tag_filters, vec!["backend"]);
    }

    #[test]
    fn test_author_filter() {
        let parsed = parse_query("rust author:zhangsan");
        assert_eq!(parsed.author_filter, Some("zhangsan".to_string()));
    }

    #[test]
    fn test_phrase_query() {
        let parsed = parse_query("\"hello world\"");
        assert_eq!(parsed.phrase_terms.len(), 1);
    }

    #[test]
    fn test_date_filter() {
        let parsed = parse_query("rust after:2026-01-01");
        assert!(parsed.after_date.is_some());
    }

    #[test]
    fn test_mixed_query() {
        let parsed = parse_query("rust 内存管理 author:张三 tag:backend after:2026-03-01");
        assert!(!parsed.search_terms.is_empty());
        assert_eq!(parsed.author_filter, Some("张三".to_string()));
        assert_eq!(parsed.tag_filters, vec!["backend"]);
        assert!(parsed.after_date.is_some());
    }
}
