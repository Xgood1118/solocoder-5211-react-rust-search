use std::path::Path;
use walkdir::WalkDir;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::Document;
use crate::markdown::markdown_to_html;
use crate::models::new_doc_id;

pub fn load_docs_from_dir(dir: &str) -> Result<Vec<Document>> {
    let mut docs = Vec::new();
    let path = Path::new(dir);

    if !path.exists() {
        return Ok(docs);
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(doc) = load_single_doc(path) {
                docs.push(doc);
            }
        }
    }

    Ok(docs)
}

fn load_single_doc(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)?;
    let (front_matter, body) = parse_front_matter(&content);

    let title = front_matter
        .get("title")
        .cloned()
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

    let author = front_matter
        .get("author")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let tags: Vec<String> = front_matter
        .get("tags")
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let created_at = front_matter
        .get("date")
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let html = markdown_to_html(&body);

    let id = new_doc_id();

    Ok(Document {
        id,
        title,
        author,
        tags,
        created_at,
        content: body,
        html,
    })
}

fn parse_front_matter(content: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut front_matter = std::collections::HashMap::new();

    if !content.starts_with("---\n") {
        return (front_matter, content.to_string());
    }

    let rest = &content[4..];
    if let Some(end_idx) = rest.find("\n---") {
        let yaml_part = &rest[..end_idx];
        let body_start = end_idx + 4;
        let body = if body_start < rest.len() {
            rest[body_start..].trim_start().to_string()
        } else {
            String::new()
        };

        for line in yaml_part.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                front_matter.insert(key, value);
            }
        }

        (front_matter, body)
    } else {
        (front_matter, content.to_string())
    }
}
