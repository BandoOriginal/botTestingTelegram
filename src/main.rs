use std::fs;
use std::path::Path;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::types::InputFile;
use serde::Deserialize;
use url::Url;
use reqwest::header::{HeaderMap, USER_AGENT};

// Nombre del archivo local para guardar la última ID procesada
const LAST_ID_FILE: &str = "last_id.txt";

// URL de la API (ajústala si necesitas tags específicos)
const API_URL: &str = "https://e621.net/posts.json?limit=20";

// Estructuras para deserializar el JSON de la API
#[derive(Debug, Deserialize)]
struct E621Response {
    posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
struct Post {
    id: i64,
    file: File,
    tags: Tags,
}

#[derive(Debug, Deserialize)]
struct File {
    url: String,
}

#[derive(Debug, Deserialize)]
struct Tags {
    artist: Vec<String>,
}

#[tokio::main]
async fn main() {
    // Cargar variables de entorno
    let token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN debe estar definido");
    let channel_id = std::env::var("CHANNEL_ID").expect("CHANNEL_ID debe estar definido");

    let bot = Bot::new(token).parse_mode(ParseMode::Html);

    // Fetch de nuevos posts
    let nuevos_posts = fetch_nuevos_posts().await;

    if nuevos_posts.is_empty() {
        println!("No hay nuevos posts.");
        return;
    }

    // Enviar cada imagen nueva
    for post in &nuevos_posts {
        let url = Url::parse(post.file.url.as_str()).expect("❌ URL inválida");
        let photo = InputFile::url(url);
        if let Err(e) = bot
            .send_photo(channel_id.clone(), photo)
            .caption(format!("🎨 Nuevo arte de: {}", post.tags.artist.join(", ")))
            .send()
            .await
        {
            eprintln!("❌ Error al enviar imagen {}: {:?}", post.id, e);
        } else {
            println!("✅ Imagen {} enviada", post.id);
        }
    }
    // Guardar la última ID procesada
    if let Some(post) = nuevos_posts.last() {
        save_last_id(&post.id.to_string()).expect("❌ No se pudo guardar la última ID");
    }
}

// Función que hace la llamada a la API
async fn fetch_nuevos_posts() -> Vec<Post> {
    let client = reqwest::Client::new();

    // Preparar headers (algunas APIs requieren User-Agent)
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "TelegramAutoPoster/1.0".parse().unwrap());

    let response = client
        .get(API_URL)
        .headers(headers)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("❌ Error al hacer GET a la API")
        .json::<E621Response>()
        .await
        .expect("❌ Error al parsear JSON");

    let last_id = read_last_id();

    // Filtrar posts nuevos
    response
        .posts
        .into_iter()
        .filter(|p| {
            last_id
                .as_ref()
                .map_or(true, |saved_id| p.id > saved_id.parse::<i64>().unwrap_or(0))
        })
        .collect()
}

// Leer la última ID desde el archivo
fn read_last_id() -> Option<String> {
    if Path::new(LAST_ID_FILE).exists() {
        fs::read_to_string(LAST_ID_FILE)
            .ok()
            .filter(|s| !s.trim().is_empty())
    } else {
        None
    }
}

// Guardar la última ID en el archivo
fn save_last_id(id: &str) -> std::io::Result<()> {
    fs::write(LAST_ID_FILE, id)
}