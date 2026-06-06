mod models;
mod tokenizer;
mod index;
mod query_parser;
mod validation;
mod markdown;
mod doc_store;
mod search_service;
mod search_history;
mod doc_loader;
mod handlers;

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;

use doc_store::DocumentStoreLock;
use index::IndexStore;
use search_history::SearchHistoryStore;
use index::InvertedIndex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let doc_store = DocumentStoreLock::new();
    let index_store = IndexStore::new();
    let history_store = SearchHistoryStore::new();

    let docs_dir = std::env::var("DOCS_DIR").unwrap_or_else(|_| "../docs".to_string());
    log::info!("Loading documents from: {}", docs_dir);

    match doc_loader::load_docs_from_dir(&docs_dir) {
        Ok(docs) => {
            log::info!("Loaded {} documents", docs.len());

            let mut store_write = doc_store.store.write();
            for doc in &docs {
                store_write.insert(doc.clone());
            }
            drop(store_write);

            let store_read = doc_store.store.read();
            let all_docs = store_read.get_all();
            let index = InvertedIndex::build(&all_docs);
            drop(store_read);

            let mut index_write = index_store.index.write();
            *index_write = index;
            drop(index_write);
        }
        Err(e) => {
            log::warn!("Failed to load docs: {}", e);
        }
    }

    let doc_data = web::Data::new(doc_store);
    let index_data = web::Data::new(index_store);
    let history_data = web::Data::new(history_store);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    log::info!("Starting server on {}", bind_addr);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(doc_data.clone())
            .app_data(index_data.clone())
            .app_data(history_data.clone())
            .service(
                web::scope("/api/docs")
                    .route("", web::get().to(handlers::docs::list_docs))
                    .route("", web::post().to(handlers::docs::create_doc))
                    .route("/batch", web::post().to(handlers::docs::batch_import))
                    .route("/{id}", web::get().to(handlers::docs::get_doc))
                    .route("/{id}", web::put().to(handlers::docs::update_doc))
                    .route("/{id}", web::delete().to(handlers::docs::delete_doc))
            )
            .route("/api/search", web::get().to(handlers::search::search))
            .route("/api/autocomplete", web::get().to(handlers::autocomplete::autocomplete))
            .service(
                web::scope("/api/history")
                    .route("", web::get().to(handlers::autocomplete::get_history))
                    .route("", web::delete().to(handlers::autocomplete::delete_history_item))
                    .route("/clear", web::post().to(handlers::autocomplete::clear_history))
            )
            .route("/api/force-rebuild", web::post().to(handlers::docs::force_rebuild))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
