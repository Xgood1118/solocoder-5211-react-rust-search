use crate::models::{ApiError, CreateDocumentRequest, UpdateDocumentRequest};

const MAX_TITLE_LEN: usize = 100;
const MAX_CONTENT_BYTES: usize = 1024 * 1024; // 1MB
const MAX_QUERY_LEN: usize = 200;

pub fn validate_create_doc(req: &CreateDocumentRequest) -> Result<(), ApiError> {
    if req.title.is_empty() {
        return Err(ApiError {
            error: "标题不能为空".to_string(),
            field: Some("title".to_string()),
        });
    }
    if req.title.chars().count() > MAX_TITLE_LEN {
        return Err(ApiError {
            error: format!("标题不能超过 {} 字", MAX_TITLE_LEN),
            field: Some("title".to_string()),
        });
    }

    if req.author.is_empty() {
        return Err(ApiError {
            error: "作者不能为空".to_string(),
            field: Some("author".to_string()),
        });
    }

    if req.content.is_empty() {
        return Err(ApiError {
            error: "正文不能为空".to_string(),
            field: Some("content".to_string()),
        });
    }
    if req.content.len() > MAX_CONTENT_BYTES {
        return Err(ApiError {
            error: "正文不能超过 1MB".to_string(),
            field: Some("content".to_string()),
        });
    }

    let mut seen = std::collections::HashSet::new();
    for tag in &req.tags {
        if !seen.insert(tag.clone()) {
            return Err(ApiError {
                error: format!("标签重复: {}", tag),
                field: Some("tags".to_string()),
            });
        }
    }

    Ok(())
}

pub fn validate_update_doc(req: &UpdateDocumentRequest) -> Result<(), ApiError> {
    if let Some(title) = &req.title {
        if title.is_empty() {
            return Err(ApiError {
                error: "标题不能为空".to_string(),
                field: Some("title".to_string()),
            });
        }
        if title.chars().count() > MAX_TITLE_LEN {
            return Err(ApiError {
                error: format!("标题不能超过 {} 字", MAX_TITLE_LEN),
                field: Some("title".to_string()),
            });
        }
    }

    if let Some(content) = &req.content {
        if content.is_empty() {
            return Err(ApiError {
                error: "正文不能为空".to_string(),
                field: Some("content".to_string()),
            });
        }
        if content.len() > MAX_CONTENT_BYTES {
            return Err(ApiError {
                error: "正文不能超过 1MB".to_string(),
                field: Some("content".to_string()),
            });
        }
    }

    if let Some(tags) = &req.tags {
        let mut seen = std::collections::HashSet::new();
        for tag in tags {
            if !seen.insert(tag.clone()) {
                return Err(ApiError {
                    error: format!("标签重复: {}", tag),
                    field: Some("tags".to_string()),
                });
            }
        }
    }

    Ok(())
}

pub fn validate_query(query: &str) -> Result<(), ApiError> {
    if query.is_empty() {
        return Err(ApiError {
            error: "查询不能为空".to_string(),
            field: Some("q".to_string()),
        });
    }
    if query.chars().count() > MAX_QUERY_LEN {
        return Err(ApiError {
            error: format!("查询不能超过 {} 字符", MAX_QUERY_LEN),
            field: Some("q".to_string()),
        });
    }
    Ok(())
}
