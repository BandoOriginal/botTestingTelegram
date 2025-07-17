/*use teloxide::prelude::*;
use teloxide::types::ParseMode;
use chrono::Local;

#[tokio::main]
async fn main() {
    // Obtiene el token desde una variable de entorno
    let token = std::env::var("TELOXIDE_TOKEN")
        .expect("La variable TELOXIDE_TOKEN debe estar definida");

    let bot = Bot::new(token).parse_mode(ParseMode::Html);

    // Intervalo cada 1 hora (3600 segundos)
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));

    // Canal donde se publicará
    let channel_id = std::env::var("CHANNEL_ID")
        .expect("La variable CHANNEL_ID debe estar definida");

    loop {
        interval.tick().await;

        let mensaje = format!("📢 Nuevo post automático a las {}", Local::now().format("%Y-%m-%d %H:%M"));
        if let Err(e) = bot
            .send_message(channel_id.clone(), mensaje)
            .send()
            .await
        {
            eprintln!("Error al enviar mensaje: {:?}", e);
        } else {
            println!("✅ Mensaje enviado a Telegram");
        }
    }
}*/
use std::env;

#[tokio::main]
async fn main() {
    // Imprimir todas las variables de entorno
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }

    // Intentar leer las variables críticas
    match env::var("TELOXIDE_TOKEN") {
        Ok(token) => println!("Token encontrado: {}", token),
        Err(e) => eprintln!("Error al leer TELOXIDE_TOKEN: {}", e),
    }

    match env::var("CHANNEL_ID") {
        Ok(id) => println!("Channel ID encontrado: {}", id),
        Err(e) => eprintln!("Error al leer CHANNEL_ID: {}", e),
    }
}
