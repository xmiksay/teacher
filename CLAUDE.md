# CLAUDE.md

## Project Overview

AI-powered language tutor — the student controls the lesson direction while Claude subtly focuses on weak points from past sessions. Multi-user, auth-gated via Bearer tokens.

**Repo:** `github.com/xmiksay/teacher` | **Deployed at:** `teacher.sourcelab.cz` (K8s, nginx ingress)

## Build & Run

### Prerequisites
- PostgreSQL via Docker: `docker compose up -d`
- Create `.env` with at minimum `ANTHROPIC_API_KEY=sk-ant-...`

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

### Docker (production)
```bash
docker build -t teacher .        # multi-stage: builds frontend, then backend, copies binaries
```
The Dockerfile builds the Vue client first, copies `client/dist` into the Rust build context (rust-embed), then produces a slim Debian image with three binaries: `teacher_server`, `teacher_cli`, `teacher_migration`.

### Environment Variables
| Variable | Default | Notes |
|---|---|---|
| `DATABASE_URL` | `postgres://teacher:teacher@localhost:5432/teacher` | |
| `LLM_PROVIDER` | `claude` | `claude` or `ollama` |
| `ANTHROPIC_API_KEY` | — | Required when `LLM_PROVIDER=claude` |
| `CLAUDE_MODEL` | `claude-sonnet-4-20250514` | |
| `OLLAMA_URL` | `http://localhost:11434` | Ollama server address |
| `OLLAMA_MODEL` | `llama3.1` | Must support tool calling |
| `SELF_URL` | `http://localhost:3000` | Used for MCP callback URLs |
| `LISTEN_ADDR` | `0.0.0.0:3000` | |

## Architecture

### Request Flow (Lesson Chat)
1. Vue SPA sends `POST /api/lesson/chat` with `{profile_id, lesson_id, messages[]}`
2. `AuthUser` extractor validates Bearer token from `auth_tokens` table
3. Backend loads profile, weak points, and LRU vocabulary from DB
4. Builds system prompt with student context baked in
5. Calls Claude API with tools defined inline (not MCP protocol — direct tool_use loop)
6. Tool calls (`add_vocabulary`, `bump_vocabulary`, `add_weak_point`, `resolve_weak_point`, `set_topic_preference`) are executed locally against the DB in a loop until Claude returns text
7. Conversation is persisted to the `lessons` table if `lesson_id` is provided

### Key Design Decisions
- **Tools are server-side, not MCP**: Despite `/mcp/*` routes existing for testing, the lesson chat handler executes tools directly via `execute_tool()` in `src/api/lesson.rs`, not via HTTP callbacks
- **LRU vocabulary**: Ordered by `last_practiced ASC`, no spaced repetition — natural rotation at scale
- **Static files embedded**: `client/dist` is compiled into the server binary via `rust-embed`, so a client build is needed before `cargo build` for production
- **Auth**: Bearer token in `Authorization` header, extracted via Axum's `FromRequestParts` (`src/auth.rs`). Passwords hashed with argon2.
- **Single binary deployment**: All static assets are embedded, the server is self-contained

### API Routes
| Method | Path | Handler | Auth |
|---|---|---|---|
| POST | `/api/auth/register` | `api::auth::register` | No |
| POST | `/api/auth/login` | `api::auth::login` | No |
| POST | `/api/lesson/chat` | `api::lesson::chat` | Yes |
| GET | `/api/lessons/{profile_id}` | `api::lesson_history::list_lessons` | Yes |
| POST | `/api/lessons` | `api::lesson_history::create_lesson` | Yes |
| GET | `/api/lessons/{id}/detail` | `api::lesson_history::get_lesson` | Yes |
| DELETE | `/api/lessons/{id}/delete` | `api::lesson_history::delete_lesson` | Yes |
| DELETE | `/api/lessons/{lesson_id}/messages/{message_id}` | `api::lesson_history::delete_message` | Yes |
| GET/POST | `/api/profiles` | `api::profile::list/create` | Yes |
| GET/PUT/DELETE | `/api/profiles/{id}` | `api::profile::get/update/delete` | Yes |
| GET | `/api/weak-points/{profile_id}` | `api::weak_points::list_weak_points` | Yes |
| POST | `/api/vocab` | `api::vocab::create_vocab` | Yes |
| GET | `/api/vocab/{profile_id}` | `api::vocab::list_vocab` | Yes |
| DELETE | `/api/vocab/{id}/delete` | `api::vocab::delete_vocab` | Yes |
| DELETE | `/api/vocab/{profile_id}/delete-all` | `api::vocab::delete_all_vocab` | Yes |

### Module Layout

**Backend (`src/`)**
- `lib.rs` — `AppState` struct (db, http_client, anthropic_api_key, claude_model, self_url)
- `auth.rs` — `AuthUser` extractor (FromRequestParts)
- `api/` — REST handlers: auth, lesson, lesson_history, profile, vocab, weak_points
- `mcp/` — MCP-style REST endpoints (separate from tool execution in lesson.rs, used for testing)
- `entities/` — SeaORM models: user, auth_token, user_language_profile, vocabulary, weak_point, lesson, lesson_message
- `migration/` — SeaORM migrations (auto-applied on server start)
- `bin/` — three binaries: teacher_server, teacher_migration, teacher_cli

**Frontend (`client/src/`)**
- `views/` — LoginView, LessonView, LessonHistoryView, VocabView, WeakPointsView, SettingsView
- `stores/` — Pinia stores: auth, lesson, profile, vocab, weakPoints
- `router.ts` — SPA routing with auth guard (redirects to `/login` if unauthenticated)
- `composables/` — shared composition functions

### Data Model
- `users` — email + argon2 password hash
- `auth_tokens` — Bearer tokens linked to users
- `user_language_profiles` — language, level (A1-C2), style, explanation_language, is_active
- `vocabulary` — word, translation, added_by (user/claude), context, last_practiced, error_count
- `weak_points` — type (grammar/vocabulary/phrase), detail, active flag
- `lessons` — conversation persistence
- `lesson_messages` — individual messages within lessons

### Deployment
- K8s manifests in `k8s.yml`: Secret, Deployment, Service, Ingress
- Docker image: `ghcr.io/euroska/teacher:master`
- Ingress: nginx with TLS via cert-manager, 600s proxy timeouts, buffering off (for streaming)
