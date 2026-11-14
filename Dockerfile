# Сервер Pixel Village. Локально: docker build -t pixel-village-server . && docker run -p 8080:8080 pixel-village-server
FROM rust:1-slim AS build
WORKDIR /app

# Встановити залежності для компіляції SQLx
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY server ./server
COPY admin.html ./admin.html
COPY client/Cargo.toml ./client/Cargo.toml
RUN mkdir -p client/src && echo 'fn main(){}' > client/src/main.rs
RUN cargo build --release -p server

FROM debian:stable-slim
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/server /usr/local/bin/server
# Хостинги (Render и др.) сами подставляют PORT; по умолчанию 8080.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
