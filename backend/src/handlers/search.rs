use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::Deserialize;
use std::time::Instant;

use crate::doc_store::DocumentStoreLock;
use crate::index::IndexStore;
use crate::search_service::SearchService;
use crate::models::{ApiError, SearchResponse};
use crate::search_history::SearchHistoryStore;
use crate::validation::validate_query;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub r#type: Option<String>,
}

pub async fn search(
    doc_data: web::Data<DocumentStoreLock>,
    index_data: web::Data<IndexStore>,
    history_data: web::Data<SearchHistoryStore>,
    query: web::Query<SearchQuery>,
    req: HttpRequest,
) -> impl Responder {
    let q = query.q.clone();

    if let Err(e) = validate_query(&q) {
        return HttpResponse::BadRequest().json(e);
    }

    let ip = req.peer_addr().map(|addr| addr.ip().to_string());

    let start = Instant::now();

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let (results, total, tokens, _display_terms) = SearchService::search(
        &doc_data,
        &index_data,
        &q,
        limit,
        offset,
    );

    let elapsed = start.elapsed();
    let took_ms = elapsed.as_millis() as u64;

    history_data.history.write().add(&q, ip);

    let response = SearchResponse {
        total,
        results,
        tokens,
        took_ms,
    };

    HttpResponse::Ok().json(response)
}
