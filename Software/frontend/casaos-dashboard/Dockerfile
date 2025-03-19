# Dockerfile corrigido
FROM rust:latest as builder

# Instala as dependências musl dentro do container
RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Configura o target musl e compila
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

# Estágio final com Alpine
FROM alpine:latest

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/casaos-dashboard /app/

EXPOSE 8080
CMD ["/app/casaos-dashboard"]
# No Dockerfile, adicione:
ENV LANG C.UTF-8
ENV LC_ALL C.UTF-8

