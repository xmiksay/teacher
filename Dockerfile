# -- Frontend build --
FROM node:25-alpine AS frontend
WORKDIR /app/client
COPY client/package.json client/package-lock.json ./
RUN npm ci
COPY client/ ./
RUN npm run build

# -- Backend build --
FROM rust:1.87-bookworm AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY --from=frontend /app/client/dist client/dist
RUN cargo build --release --bin teacher_server

# -- Runtime --
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/target/release/teacher_server /usr/local/bin/teacher_server
EXPOSE 3000
CMD ["teacher_server"]
