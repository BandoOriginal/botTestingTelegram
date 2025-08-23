use std::time::Duration;
use axum::{Router, routing::get, response::IntoResponse};
use axum::Server; // <-- Import necesario para axum 0.7
use reqwest::Url;
use serde::Deserialize;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, InputFile};
use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

#[derive(Debug, Deserialize)]
struct E621Response { posts: Vec<Post> }

#[derive(Debug, Deserialize, Clone)]
struct Post { id: i64, file: File, tags: Tags }

#[derive(Debug, Deserialize, Clone)]
struct File { #[serde(default)] url: Option<String> }

#[derive(Debug, Deserialize, Clone)]
struct Tags { artist: Option<Vec<String>> }

const SOURCE_NAME: &str = "e621";

#[tokio::main]
async fn main() -> Result<()> {
    // Pool de Postgres
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await?;

    // Crear tabla si no existe
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS last_id_tracker (
            source_name TEXT PRIMARY KEY,
            last_post_id BIGINT NOT NULL,
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        );"
    )
    .execute(&pool).await?;

    // Servidor HTTP para cron-job.org
    let pool_clone = pool.clone();
    let app = Router::new().route("/run", get(move || run_job_handler(pool_clone.clone())));
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port).parse()?;
    println!("🚀 Server listening on {}", addr);

    // ⚡ Cambio principal: usar Server::bind
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn run_job_handler(pool: PgPool) -> impl IntoResponse {
    match run_job(pool).await {
        Ok(msg) => msg,
        Err(e) => format!("❌ Error: {:?}", e),
    }
}

async fn run_job(pool: PgPool) -> Result<String> {
    let token = std::env::var("TELOXIDE_TOKEN")?;
    let channel_id = std::env::var("CHANNEL_ID")?;
    let bot = Bot::new(token).parse_mode(ParseMode::Html);

    // Leer la última ID desde DB
    let last_id: Option<i64> = sqlx::query_scalar(
        "SELECT last_post_id FROM last_id_tracker WHERE source_name = $1"
    )
    .bind(SOURCE_NAME)
    .fetch_optional(&pool).await?;

    let api_url = if let Some(id) = last_id {
        format!("https://e621.net/posts.json?tags=femboy+rating:s+order:id_desc&page=a{}", id)
    } else {
        format!("https://e621.net/posts.json?tags=femboy+rating:s+order:id_desc&page=a10000000")
    };

    let response = reqwest::Client::new()
        .get(&api_url)
        .header("User-Agent", "TelegramAutoPoster/1.0")
        .timeout(Duration::from_secs(10))
        .send().await?
        .json::<E621Response>().await?;

    if response.posts.is_empty() {
        return Ok("No hay nuevos posts.".to_string());
    }

    let mut max_id = last_id.unwrap_or(0);

    for post in &response.posts {
        if let Some(url_str) = &post.file.url {
            if let Ok(url) = Url::parse(url_str) {
                let photo = InputFile::url(url);
                let artist = post.tags.artist.as_ref().map_or("Unknown".to_string(), |a| a.join(", "));
                if let Err(e) = bot.send_photo(channel_id.clone(), photo)
                    .caption(format!("🎨 Nuevo arte de: {}", artist))
                    .send().await
                {
                    eprintln!("❌ Error al enviar imagen {}: {:?}", post.id, e);
                } else {
                    println!("✅ Imagen {} enviada", post.id);
                }
            }
        }
        if post.id > max_id { max_id = post.id; } // Guardamos siempre la mayor ID
    }

    // Guardar la última ID en DB
    sqlx::query(
        "INSERT INTO last_id_tracker (source_name, last_post_id)
         VALUES ($1, $2)
         ON CONFLICT (source_name)
         DO UPDATE SET last_post_id = $2, updated_at = NOW()"
    )
    .bind(SOURCE_NAME)
    .bind(max_id)
    .execute(&pool).await?;

    Ok(format!("✅ {} posts procesados", response.posts.len()))
}
