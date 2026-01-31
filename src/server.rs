use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::flashcards::FlashcardList;
use crate::kaikki::get_from_kaikki;

#[derive(Clone)]
struct AppState {
    books_dir: String,
}

#[derive(Serialize)]
struct Book {
    name: String,
    path: String,
}

#[derive(Deserialize)]
struct AddFlashcardRequest {
    word: String,
    definition: String,
    context: Option<String>,
    source: Option<String>,
}

#[derive(Deserialize)]
struct ReviewFlashcardRequest {
    card_id: String,
    quality: u8,
}

pub async fn start_server(port: u16, dir: String) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState {
        books_dir: dir.clone(),
    });

    let app = Router::new()
        .route("/api/books", get(list_books))
        .route("/api/books/*path", get(get_book_content))
        .route("/api/lookup/:word", get(lookup_word))
        .route("/api/flashcards", get(get_flashcards).post(add_flashcard))
        .route("/api/flashcards/review", post(review_flashcard))
        .nest_service("/", ServeDir::new("assets"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_books(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut books = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&state.books_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ["epub", "pdf", "txt"].contains(&ext) {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        books.push(Book {
                            name: name.to_string(),
                            path: name.to_string(),
                        });
                    }
                }
            }
        }
    }
    Json(books)
}

async fn get_book_content(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let base_path = match std::fs::canonicalize(&state.books_dir) {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Invalid books directory").into_response(),
    };

    let file_path = std::path::Path::new(&state.books_dir).join(&path);

    // Resolve symlinks and .. to ensure we don't traverse up
    let canonical_path = match std::fs::canonicalize(&file_path) {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    if !canonical_path.starts_with(&base_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // Determine content type
    let mime_type = match canonical_path.extension().and_then(|s| s.to_str()) {
        Some("epub") => "application/epub+zip",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        _ => "application/octet-stream",
    };

    match std::fs::read(&canonical_path) {
        Ok(bytes) => (
            [(axum::http::header::CONTENT_TYPE, mime_type)],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
    }
}

async fn lookup_word(Path(word): Path<String>) -> impl IntoResponse {
    match get_from_kaikki(&word).await {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to lookup word: {}", e),
        )
            .into_response(),
    }
}

async fn get_flashcards() -> impl IntoResponse {
    match FlashcardList::load("flashcards.json") {
        Ok(list) => Json(list.cards).into_response(),
        Err(_) => Json(Vec::<crate::flashcards::Flashcard>::new()).into_response(),
    }
}

async fn add_flashcard(Json(payload): Json<AddFlashcardRequest>) -> impl IntoResponse {
    let mut list = FlashcardList::load("flashcards.json").unwrap_or_else(|_| FlashcardList::new());
    list.add(
        payload.word,
        payload.definition,
        payload.context,
        payload.source,
    );

    match list.save("flashcards.json") {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save flashcard: {}", e),
        )
            .into_response(),
    }
}

async fn review_flashcard(Json(payload): Json<ReviewFlashcardRequest>) -> impl IntoResponse {
    let mut list = FlashcardList::load("flashcards.json").unwrap_or_else(|_| FlashcardList::new());
    
    if list.review_card(&payload.card_id, payload.quality).is_some() {
         match list.save("flashcards.json") {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save flashcard: {}", e),
            ).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Card not found").into_response()
    }
}
