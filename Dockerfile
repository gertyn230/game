# Сервер Pixel Village. Локально: docker build -t pixel-village-server . && docker run -p 8080:8080 pixel-village-server
FROM rust:1-slim AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY server ./server
COPY client/Cargo.toml ./client/Cargo.toml
RUN mkdir -p client/src && echo 'fn main(){}' > client/src/main.rs
RUN cargo build --release -p server

FROM debian:stable-slim
COPY --from=build /app/target/release/server /usr/local/bin/server
# Хостинги (Render и др.) сами подставляют PORT; по умолчанию 8080.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
