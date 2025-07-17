# Usamos una imagen oficial de Rust
FROM rust:1.70-slim

# Instalamos dependencias del sistema necesarias
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Directorio de trabajo dentro del contenedor
WORKDIR /usr/src/app

# Copiamos los archivos de configuración primero
COPY Cargo.toml Cargo.lock ./

# Descargamos las dependencias (sin compilar aún)
RUN cargo fetch

# Copiamos el código fuente
COPY src ./src

# Compilamos el proyecto en modo release
RUN cargo build --release

# Comando para ejecutar tu bot cuando se inicie el contenedor
CMD ["./target/release/telegram_auto_poster"]
