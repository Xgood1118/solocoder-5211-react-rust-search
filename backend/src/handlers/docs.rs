use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::Deserialize;
use rayon::prelude::*;

use crate::doc_store::DocumentStoreLock;
use crate::index::IndexStore;
use crate::models::{
    ApiError, BatchFailure, BatchImportResult, CreateDocumentRequest, Document,
    UpdateDocumentRequest,
};
use crate::validation::{validate_create_doc, validate_update_doc};
use crate::search_history::SearchHistoryStore;
use crate::markdown::markdown_to_html;
use crate::models::new_doc_id;
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct DocListQuery {
    pub tag: Option<Vec<String>>,
    pub author: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_docs(
    data: web::Data<DocumentStoreLock>,
    query: web::Query<DocListQuery>,
) -> impl Responder {
    let store = data.store.read();
    let docs: Vec<Document> = match (&query.tag, &query.author) {
        (Some(tags), Some(author)) => store.filter_by_tags_and_author(tags, author),
        (Some(tags), None) => store.filter_by_tags(tags),
        (None, Some(author)) => store.filter_by_author(author),
        (None, None) => store.get_all(),
    };

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let paginated: Vec<Document> = docs.into_iter().skip(offset).take(limit).collect();

    HttpResponse::Ok().json(paginated)
}

pub async fn get_doc(
    data: web::Data<DocumentStoreLock>,
    id: web::Path<String>,
) -> impl Responder {
    let store = data.store.read();
    match store.get_by_id(&id) {
        Some(doc) => HttpResponse::Ok().json(doc),
        None => HttpResponse::NotFound().json(ApiError {
            error: "文档不存在".to_string(),
            field: None,
        }),
    }
}

pub async fn create_doc(
    data: web::Data<DocumentStoreLock>,
    index_data: web::Data<IndexStore>,
    body: web::Json<CreateDocumentRequest>,
) -> impl Responder {
    if let Err(e) = validate_create_doc(&body) {
        return HttpResponse::BadRequest().json(e);
    }

    let doc = {
        let store = data.store.read();
        store.create_from_request(body.into_inner())
    };

    let doc_clone = doc.clone();
    data.store.write().insert(doc.clone());
    index_data.index.write().add_document(&doc_clone);

    HttpResponse::Created().json(doc)
}

pub async fn update_doc(
    data: web::Data<DocumentStoreLock>,
    index_data: web::Data<IndexStore>,
    id: web::Path<String>,
    body: web::Json<UpdateDocumentRequest>,
) -> impl Responder {
    if let Err(e) = validate_update_doc(&body) {
        return HttpResponse::BadRequest().json(e);
    }

    let doc_id = id.into_inner();

    let old_exists = data.store.read().get_by_id(&doc_id).is_some();
    if !old_exists {
        return HttpResponse::NotFound().json(ApiError {
            error: "文档不存在".to_string(),
            field: None,
        });
    }

    let mut index_write = index_data.index.write();
    index_write.remove_document(&doc_id);
    drop(index_write);

    let updated = data.store.write().update(&doc_id, body.into_inner());

    if let Some(doc) = &updated {
        index_data.index.write().add_document(doc);
    }

    match updated {
        Some(doc) => HttpResponse::Ok().json(doc),
        None => HttpResponse::NotFound().json(ApiError {
            error: "文档不存在".to_string(),
            field: None,
        }),
    }
}

pub async fn delete_doc(
    data: web::Data<DocumentStoreLock>,
    index_data: web::Data<IndexStore>,
    id: web::Path<String>,
) -> impl Responder {
    let doc_id = id.into_inner();
    let deleted = data.store.write().delete(&doc_id);

    if deleted {
        index_data.index.write().remove_document(&doc_id);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().json(ApiError {
            error: "文档不存在".to_string(),
            field: None,
        })
    }
}

pub async fn batch_import(
    data: web::Data<DocumentStoreLock>,
    index_data: web::Data<IndexStore>,
    body: web::Json<Vec<CreateDocumentRequest>>,
) -> impl Responder {
    let requests = body.into_inner();
    let mut successes: Vec<Document> = Vec::new();
    let mut failures: Vec<BatchFailure> = Vec::new();

    let chunk_size = 100;
    let chunks: Vec<&[CreateDocumentRequest]> = requests.chunks(chunk_size).collect();

    for chunk in chunks {
        let chunk_results: Vec<(usize, Result<Document, String>)> = chunk
            .par_iter()
            .enumerate()
            .map(|(i, req)| {
                let global_idx = i;
                match validate_create_doc(req) {
                    Err(e) => (global_idx, Err(e.error)),
                    Ok(_) => {
                        let html = markdown_to_html(&req.content);
                        let doc = Document {
                            id: new_doc_id(),
                            title: req.title.clone(),
                            author: req.author.clone(),
                            tags: req.tags.clone(),
                            created_at: Utc::now(),
                            content: req.content.clone(),
                            html,
                        };
                        (global_idx, Ok(doc))
                    }
                }
            })
            .collect();

        let mut doc_batch: Vec<Document> = Vec::new();
        for (i, result) in chunk_results {
            match result {
                Ok(doc) => {
                    doc_batch.push(doc);
                }
                Err(reason) => {
                    failures.push(BatchFailure { index: i, reason });
                }
            }
        }

        let mut store_write = data.store.write();
        let mut index_write = index_data.index.write();
        for doc in &doc_batch {
            store_write.insert(doc.clone());
            index_write.add_document(doc);
        }
        drop(store_write);
        drop(index_write);

        successes.extend(doc_batch);
    }

    let result = BatchImportResult {
        success_count: successes.len(),
        fail_count: failures.len(),
        failures,
    };

    HttpResponse::Ok().json(result)
}

pub async fn force_rebuild(
    data: web::Data<DocumentStoreLock>,
    index_data: web::Data<IndexStore>,
) -> impl Responder {
    use crate::index::InvertedIndex;

    let docs = data.store.read().get_all();
    let new_index = InvertedIndex::build(&docs);

    let mut index_write = index_data.index.write();
    *index_write = new_index;

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "doc_count": data.store.read().len()
    }))
}
