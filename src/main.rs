use std::fs;
use std::path::Path;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::types::InputFile;
use reqwest::Url;
use serde::Deserialize;

// Ruta del archivo que guarda la última ID procesada
const LAST_ID_FILE: &str = "last_id.txt";

// URL de la API de e621 (ajústala según tus necesidades)
const API_URL: &str = "https://e621.net/posts.json?tags=femboy+order:id_desc&limit=20";

#[derive(Debug, Deserialize)]
struct E621Response {
    posts: Vec<Post>,
}

#[derive(Debug, Deserialize, Clone)]
struct Post {
    id: i64,
    file: File,
    tags: Tags,
}

#[derive(Debug, Deserialize, Clone)]
struct File {
    url: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Tags {
    artist: Option<Vec<String>>,
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
        let artist = post.tags.artist.as_ref().map_or("Unknown".to_string(), |a| a.join(", "));
        let url = Url::parse(post.file.url.as_str()).expect("❌ URL inválida");
        let photo = InputFile::url(url);
        if let Err(e) = bot
            .send_photo(channel_id.clone(), photo)
            .caption(format!("🎨 Nuevo arte de: {}", artist))
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

// Función que llama a la API y filtra los nuevos posts
async fn fetch_nuevos_posts() -> Vec<Post> {
    let client = reqwest::Client::new();

    let response = client
        .get(API_URL)
        .header("User-Agent", "TelegramAutoPoster/1.0")
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
