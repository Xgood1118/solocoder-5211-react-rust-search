use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::search_history::SearchHistoryStore;
use crate::models::{AutocompleteResult, ApiError};

#[derive(Debug, Deserialize)]
pub struct AutocompleteQuery {
    pub q: String,
    pub limit: Option<usize>,
}

pub async fn autocomplete(
    data: web::Data<SearchHistoryStore>,
    query: web::Query<AutocompleteQuery>,
) -> impl Responder {
    if query.q.is_empty() {
        return HttpResponse::BadRequest().json(ApiError {
            error: "查询前缀不能为空".to_string(),
            field: Some("q".to_string()),
        });
    }

    let limit = query.limit.unwrap_or(10);
    let results = data.history.read().autocomplete(&query.q, limit);

    let response: Vec<AutocompleteResult> = results
        .into_iter()
        .map(|(query, count)| AutocompleteResult { query, count })
        .collect();

    HttpResponse::Ok().json(response)
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

pub async fn get_history(
    data: web::Data<SearchHistoryStore>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50);
    let entries = data.history.read().list(limit);
    HttpResponse::Ok().json(entries)
}

pub async fn delete_history_item(
    data: web::Data<SearchHistoryStore>,
    query: web::Query<DeleteHistoryQuery>,
) -> impl Responder {
    data.history.write().delete(&query.q);
    HttpResponse::NoContent().finish()
}

#[derive(Debug, Deserialize)]
pub struct DeleteHistoryQuery {
    pub q: String,
}

pub async fn clear_history(
    data: web::Data<SearchHistoryStore>,
) -> impl Responder {
    data.history.write().clear();
    HttpResponse::NoContent().finish()
}
