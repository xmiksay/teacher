# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AI-powered language tutor where the student controls the lesson direction while Claude subtly focuses on weak points from past sessions. Multi-user, auth-gated.

## Build & Run

### Prerequisites
- PostgreSQL via Docker: `docker compose up -d`
- Copy `.env.example` to `.env` and set `ANTHROPIC_API_KEY`

### Backend (Rust/Axum)
```bash
cargo build                      # compile
cargo run --bin teacher_server   # starts on 0.0.0.0:3000, auto-runs migrations
cargo run --bin teacher_migration up|down|status [steps]
cargo run --bin teacher_cli profiles|vocab|weak-points [profile_id]
cargo check                      # quick compile check after changes
```

### Frontend (Vue 3 + Vite)
```bash
cd client
nvm use
npm install
npm run dev          # dev server on :5173, proxies /api to :3000
npm run build        # outputs to client/dist (embedded into server binary via rust-embed)
```

### Environment Variables
- `DATABASE_URL` — defaults to `postgres://teacher:teacher@localhost:5432/teacher`
- `ANTHROPIC_API_KEY` — required
- `CLAUDE_MODEL` — defaults to `claude-sonnet-4-20250514`
- `SELF_URL` — defaults to `http://localhost:3000` (used for MCP callback URLs)
- `LISTEN_ADDR` — defaults to `0.0.0.0:3000`

## Architecture

### Request Flow (Lesson Chat)
1. Vue SPA sends `POST /api/lesson/chat` with `{profile_id, lesson_id, messages[]}`
2. `AuthUser` extractor validates Bearer token from `auth_tokens` table
3. Backend loads profile, weak points, and LRU vocabulary from DB
4. Builds system prompt with student context baked in
5. Calls Claude API with tools defined inline (not MCP protocol — direct tool_use loop)
6. Tool calls (add_vocabulary, bump_vocabulary, add_weak_point, resolve_weak_point, set_topic_preference) are executed locally against the DB in a loop until Claude returns text
7. Conversation is persisted to the `lessons` table if `lesson_id` is provided

### Key Design Decisions
- **Tools are server-side, not MCP**: Despite `/mcp/*` routes existing for testing, the lesson chat handler executes tools directly via `execute_tool()` in `src/api/lesson.rs`, not via HTTP callbacks
- **LRU vocabulary**: Ordered by `last_practiced ASC`, no spaced repetition — natural rotation at scale
- **Static files embedded**: `client/dist` is compiled into the server binary via `rust-embed`, so a client build is needed before `cargo build` for production
- **Auth**: Bearer token in `Authorization` header, extracted via Axum's `FromRequestParts` (`src/auth.rs`)

### Module Layout
- `src/api/` — REST handlers (auth, lesson, lesson_history, profile, vocab, weak_points)
- `src/mcp/` — MCP-style REST endpoints (separate from the tool execution in lesson.rs)
- `src/entities/` — SeaORM entity definitions (user, auth_token, user_language_profile, vocabulary, weak_point, lesson)
- `src/migration/` — SeaORM migrations (auto-applied on server start)
- `src/auth.rs` — `AuthUser` extractor
- `src/bin/` — three binaries: server, migration CLI, data inspection CLI
- `client/src/stores/` — Pinia stores (auth, lesson, profile, vocab, weakPoints)
- `client/src/views/` — Vue views (Login, Lesson, LessonHistory, Vocab, WeakPoints, Settings)
