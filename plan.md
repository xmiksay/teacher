# Language Learning App — Plan

## Concept

AI-powered language tutor where the student controls the direction of the lesson, while Claude subtly focuses on weak points derived from past sessions. Designed to be multi-user and public.

---

## Stack

| Layer | Technology |
|---|---|
| Frontend | Vue (SPA) |
| Backend | Rust (Axum, SeaORM) |
| Database | PostgreSQL |
| MCP Server | Rust (rmcp crate) |
| LLM | Anthropic API (Claude) |

---

## Architecture

```
Vue (SPA)
  ↓ REST
Axum
  ├── /api/lesson     → assembles system prompt + calls Claude API
  ├── /api/vocab      → CRUD for vocabulary (SeaORM → Postgres)
  └── MCP server      ← Claude accesses student context per request
        ├── get_profile
        ├── add_vocabulary
        ├── bump_vocabulary
        ├── add_weak_point
        ├── resolve_weak_point
        └── set_topic_preference
```

### Claude API Request Structure

```
POST https://api.anthropic.com/v1/messages
Header: anthropic-beta: mcp-client-2025-11-20

{
  system: <template filled with MCP context>,
  messages: [full conversation history],
  mcp_servers: [{ url, name, authorization_token }],
  tools: [{ type: "mcp_toolset", mcp_server_name: "..." }]
}
```

---

## Data Model

```sql
user_language_profile (
  id, user_id, language,
  level,               -- A1-C2
  style,               -- tutor style: friendly | strict | ...
  explanation_language -- cs | en | target
)

vocabulary (
  id, user_language_profile_id,
  word, translation,
  added_by,            -- 'user' | 'claude'
  context,             -- sentence where mistake was caught
  last_practiced,      -- for LRU ordering
  error_count
)

weak_points (
  id, user_language_profile_id,
  type,                -- grammar | vocabulary | phrase
  detail,              -- e.g. "subjuntivo", "ser vs estar"
  active               -- true | false
)
```

---

## Vocabulary System

- **LRU queue** — ordered by `last_practiced ASC`
- Mistake → `bump_vocabulary` (reset `last_practiced` to push word to front)
- No spaced repetition algorithm needed — natural rotation at scale (10k+ words)
- Flashcard UI is standalone, no LLM required
- Claude adds words silently when student makes a mistake or asks about a word

---

## MCP Tools

| Tool | Trigger |
|---|---|
| `get_profile` | Start of every lesson |
| `add_vocabulary` | Student asks about a word / makes a lexical mistake |
| `bump_vocabulary` | Student repeats a mistake on known word |
| `add_weak_point` | Claude notices recurring pattern (silent or on request) |
| `resolve_weak_point` | Student consistently uses form correctly |
| `set_topic_preference` | Student requests to include/exclude a topic |

---

## System Prompt

- Written in **English** (best token efficiency and instruction accuracy)
- **Markdown template** with placeholders filled from MCP at request time
- One template per lesson mode (conversation, grammar, vocabulary)
- Sent with every API request — Claude has no memory between requests

```
You are a language tutor for {{target_language}}.
Student level: {{level}}
Explanation language: {{explanation_language}}
Tutor style: {{style}}
Weak points: {{weak_points}}
Excluded topics: {{excluded_topics}}
```

---

## Lesson Flow

1. Student selects language + mode in UI
2. Backend loads `user_language_profile` from DB
3. Template is filled with fresh MCP context
4. Student sends message → backend appends to history → calls Claude API
5. Claude responds (markdown) + optionally calls MCP tools silently
6. Frontend renders markdown response
7. Repeat from step 4 — each request carries full conversation history

---

## Student Controls

- Target language and level
- Tutor style (friendly, strict, immersive...)
- Explanation language (native / target)
- Preferred and excluded topics
- Can change any setting mid-conversation (Claude updates MCP, persists immediately)

---

## Error Correction Format

Corrections returned as markdown inline in Claude's response:

**Original:** ¿De donde esta la biblioteca?
**Corrected:** ¿Dónde **está** la biblioteca?

**Mistakes:**
1. `De donde` → `Dónde` — ...
2. `esta` → `está` — ...

---

## Out of Scope (for now)

- Auth / user management
- Rate limiting and cost control for public users
- Analytics / progress reports
- FSRS or spaced repetition algorithms
