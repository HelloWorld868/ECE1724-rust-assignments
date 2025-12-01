use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path as FsPath;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::TcpListener;

// Represents a song in the personal music library
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Song {
    id: u64,
    title: String,
    artist: String,
    genre: String,
    play_count: u64,
}

// Used when returning an error message as JSON
#[derive(Debug, Serialize)]
struct ErrorMessage {
    error: &'static str,
}

// Structure for receiving a new song request from POST JSON
#[derive(Debug, Deserialize)]
struct NewSongRequest {
    title: String,
    artist: String,
    genre: String,
}

// Structure for receiving search query parameters
#[derive(Debug, Deserialize)]
struct SongSearchQuery {
    title: Option<String>,
    artist: Option<String>,
    genre: Option<String>,
}

// Global shared application state
#[derive(Debug)]
struct AppState {
    visit_count: AtomicUsize,
    songs: RwLock<Vec<Song>>,
}

// Basic welcome page
async fn handle_root() -> &'static str {
    "Welcome to the Rust-powered web server!"
}

// Increments and returns the global visit counter
async fn handle_count(State(state): State<Arc<AppState>>) -> String {
    let prev = state.visit_count.fetch_add(1, Ordering::SeqCst);
    let current = prev + 1;
    format!("Visit count: {}", current)
}

// Add a new song to the library
async fn handle_songs_new(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewSongRequest>,
) -> (StatusCode, Json<Song>) {
    let mut songs = state.songs.write();

    let new_id = match songs.last() {
        Some(song) => song.id + 1,
        None => 1,
    };

    let new_song = Song {
        id: new_id,
        title: payload.title,
        artist: payload.artist,
        genre: payload.genre,
        play_count: 0,
    };

    songs.push(new_song.clone());

    if let Ok(json) = serde_json::to_string(&*songs) {
        let _ = fs::write("songs.json", json);
    }

    (StatusCode::OK, Json(new_song))
}

// Search for songs by title/artist/genre
async fn handle_songs_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SongSearchQuery>,
) -> Json<Vec<Song>> {
    let songs = state.songs.read();

    let title_filter = query.title.as_ref().map(|s| s.to_lowercase());
    let artist_filter = query.artist.as_ref().map(|s| s.to_lowercase());
    let genre_filter = query.genre.as_ref().map(|s| s.to_lowercase());

    let results: Vec<Song> = songs
        .iter()
        .cloned()
        .filter(|song| {
            let title = song.title.to_lowercase();
            let artist = song.artist.to_lowercase();
            let genre = song.genre.to_lowercase();

            // Apply title filter if provided
            if let Some(ref filter) = title_filter {
                if !title.contains(filter) {
                    return false;
                }
            }

            // Apply artist filter if provided
            if let Some(ref filter) = artist_filter {
                if !artist.contains(filter) {
                    return false;
                }
            }

            // Apply genre filter if provided
            if let Some(ref filter) = genre_filter {
                if !genre.contains(filter) {
                    return false;
                }
            }

            true
        })
        .collect();

    Json(results)
}

// Play a song by ID
async fn handle_songs_play(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<Song>, (StatusCode, Json<ErrorMessage>)> {
    let mut songs = state.songs.write();

    if let Some(idx) = songs.iter().position(|s| s.id == id) {
        songs[idx].play_count += 1;

        let song_return = songs[idx].clone();

        // Save updated song list to disk
        if let Ok(json) = serde_json::to_string(&*songs) {
            let _ = fs::write("songs.json", json);
        }

        return Ok(Json(song_return));
    }

    Err((
        StatusCode::OK,
        Json(ErrorMessage {
            error: "Song not found",
        }),
    ))
}

#[tokio::main]
async fn main() {
    // Load songs from disk (if file exists)
    let songs = {
        let path = "songs.json";

        if !FsPath::new(path).exists() {
            Vec::new()
        } else if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        }
    };

    // Build shared global state for handlers
    let state = Arc::new(AppState {
        visit_count: AtomicUsize::new(0),
        songs: RwLock::new(songs),
    });

    // Define all routes in the application
    let app = Router::new()
        .route("/", get(handle_root)) // GET /
        .route("/count", get(handle_count)) // GET /count
        .route("/songs/new", post(handle_songs_new)) // POST /songs/new
        .route("/songs/search", get(handle_songs_search)) // GET /songs/search
        .route("/songs/play/:id", get(handle_songs_play)) // GET /songs/play/ID
        .with_state(state);

    // Bind the server to localhost:8080
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("The server is currently listening on localhost:8080.");

    // Start the Axum server
    axum::serve(listener, app).await.unwrap();
}
