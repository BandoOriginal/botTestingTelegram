use teloxide::prelude::*;
use teloxide::types::ParseMode;
use chrono::Local;

#[tokio::main]
async fn main() {
    // Obtiene el token desde una variable de entorno
    let token = std::env::var("TELOXIDE_TOKEN")
        .expect("La variable TELOXIDE_TOKEN debe estar definida");

    let bot = Bot::new(token).parse_mode(ParseMode::Html);

    // Canal donde se publicará
    let channel_id = std::env::var("CHANNEL_ID")
        .expect("La variable CHANNEL_ID debe estar definida");

    // Intervalo cada 1 hora (3600 segundos)
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));

    let channel_id = std::env::var("CHANNEL_ID").expect("...");

    loop {
        interval.tick().await;

        let mensaje = format!("📢 Nuevo post automático a las {}", Local::now().format("%Y-%m-%d %H:%M"));
        if let Err(e) = bot
            .send_message(channel_id, mensaje)
            .send()
            .await
        {
            eprintln!("Error al enviar mensaje: {:?}", e);
        } else {
            println!("✅ Mensaje enviado a Telegram");
        }
    }
}