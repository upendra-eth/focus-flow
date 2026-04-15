# FocusFlow — ADHD Productivity App

FocusFlow is an AI-powered productivity app designed specifically for ADHD users. It emphasizes voice-first input, dopamine-driven engagement on the home screen, and progressive profiling that adapts to you over time rather than dumping long questionnaires up front.

## Features

- **Voice-first input** — Ramble freely; the AI classifies intent (task, journal, profile answer, and more).
- **Dopamine home screen widget** — Surfaces wins and streaks, not a guilt list of open to-dos.
- **Drip-feed profiling** — One or two questions at a time, shown in context (for example after a win).
- **Weekly AI insights** — Behavioral patterns plus concrete, actionable recommendations.
- **Offline-first with real-time sync** — Local persistence on device with background sync and push-style updates when online.

## Architecture

The system is documented end-to-end in the parent repository: **[`../ADHD_APP_ARCHITECTURE.md`](../ADHD_APP_ARCHITECTURE.md)** (system diagram, data flow voice → AI → storage → UI, database design, AI pipeline, profiling engine, widget behavior, permission UX, roadmap, and API appendix).

At a glance: an **Android** client (Compose, Room, WorkManager, Glance widget) talks to a **Rust Axum** API gateway. **PostgreSQL** holds users, tasks, profiling, and signals; **Redis** caches widget and session-style state; **Qdrant** stores embeddings for contextual retrieval; **NATS JetStream** carries domain events for async AI and fan-out. The AI path uses **Whisper** for transcription and **LLMs** for classification and weekly insights.

## Tech Stack

| Layer | Choices |
|--------|---------|
| **Backend** | Rust (Axum), PostgreSQL, Redis, Qdrant, NATS JetStream |
| **AI** | OpenAI Whisper (STT), GPT-4o-mini (classification), GPT-4o (insights; Anthropic optional via env for future paths) |
| **Mobile** | Android (Jetpack Compose, Glance widget, Room) |

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust **1.77+** (stable)
- Android Studio **Hedgehog** or newer with **SDK 34**
- OpenAI API key (and optional keys per `.env.example`)

### Backend setup

```bash
cd focusflow
cp .env.example .env
# Edit .env with your API keys and secrets

# Start infrastructure (Postgres, Redis, Qdrant, NATS)
docker compose up -d

# Run SQL migrations (from repo root, with DATABASE_URL set)
psql "$DATABASE_URL" -f migrations/001_initial_schema.sql
psql "$DATABASE_URL" -f migrations/002_seed_profiling_questions.sql

# Or use the helper script (loads .env if present)
chmod +x scripts/run_migrations.sh
./scripts/run_migrations.sh

# Build and run the API
cd backend
cargo run -p focusflow-api
```

The HTTP server listens on `SERVER_HOST` / `SERVER_PORT` (defaults `0.0.0.0:8080`).

### Android setup

1. Open the `android/` directory in Android Studio.
2. Adjust **`API_BASE_URL`** in `android/app/build.gradle.kts` if your backend is not on the emulator default (`http://10.0.2.2:8080` for the Android emulator).
3. Build and run on an emulator or physical device.

## Project structure

Directory layout (aligned with [Appendix C in `../ADHD_APP_ARCHITECTURE.md`](../ADHD_APP_ARCHITECTURE.md); SQL migrations live at the **repository root** in `migrations/`).

```
focusflow/
├── backend/                      # Rust workspace
│   ├── Cargo.toml
│   ├── api/                      # Axum HTTP + WebSocket server
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/
│   │       ├── middleware/
│   │       └── ws/
│   ├── core/                     # Domain logic
│   │   └── src/
│   │       ├── tasks/
│   │       ├── profiling/        # State machine + scheduling
│   │       ├── insights/
│   │       └── signals/
│   ├── ai/                       # AI service integrations
│   │   └── src/
│   │       ├── whisper.rs
│   │       ├── classifier.rs
│   │       ├── embeddings.rs
│   │       └── insights_generator.rs
│   └── db/                       # Database layer
│       └── src/
│           ├── postgres.rs
│           ├── qdrant.rs
│           └── redis.rs
│
├── android/                      # Android app
│   ├── app/
│   │   └── src/main/
│   │       ├── java/.../focusflow/
│   │       │   ├── ui/
│   │       │   │   ├── screens/
│   │       │   │   ├── components/
│   │       │   │   └── widget/   # Glance widget
│   │       │   ├── data/
│   │       │   │   ├── local/    # Room DB
│   │       │   │   ├── remote/   # API client
│   │       │   │   └── sync/     # Sync engine
│   │       │   ├── domain/
│   │       │   │   ├── models/
│   │       │   │   └── usecases/
│   │       │   └── voice/        # Audio recording
│   │       └── res/
│   └── build.gradle.kts
│
├── migrations/                   # PostgreSQL migrations (run in order)
├── docs/                         # API and other docs
├── scripts/                      # Dev and ops helpers
├── docker-compose.yml
├── .env.example
└── README.md
```

## API endpoints

All MVP HTTP routes and the WebSocket are listed in **Appendix B** of [`../ADHD_APP_ARCHITECTURE.md`](../ADHD_APP_ARCHITECTURE.md). Detailed request and response shapes are in **[`docs/API.md`](docs/API.md)**.

| Method | Path |
|--------|------|
| `POST` | `/api/v1/auth/device` |
| `POST` | `/api/v1/voice/upload` |
| `GET` | `/api/v1/tasks` |
| `POST` | `/api/v1/tasks` |
| `PATCH` | `/api/v1/tasks/{id}` |
| `GET` | `/api/v1/widget/state` |
| `GET` | `/api/v1/profile/next-question` |
| `POST` | `/api/v1/profile/answer` |
| `POST` | `/api/v1/profile/skip` |
| `GET` | `/api/v1/insights/latest` |
| `POST` | `/api/v1/signals` |
| `WS` | `/api/v1/ws` |

## Development

### Running tests

```bash
cd backend && cargo test
```

### Database migrations

Migration files live in **`migrations/`**. Apply them in lexical order (`001_…`, then `002_…`, and so on). Use `psql` or `./scripts/run_migrations.sh` from the `focusflow` directory.

### Environment variables

See **`.env.example`** in this directory. Summary:

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis connection URL |
| `QDRANT_URL` | Qdrant HTTP endpoint |
| `NATS_URL` | NATS server URL (JetStream) |
| `OPENAI_API_KEY` | OpenAI API key (Whisper, chat, embeddings) |
| `ANTHROPIC_API_KEY` | Optional; reserved for alternate insight paths |
| `JWT_SECRET` | Secret for signing device JWTs |
| `RUST_LOG` | Tracing filter (e.g. `focusflow_api=debug`) |
| `SERVER_HOST` / `SERVER_PORT` | API bind address |
| `WHISPER_MODEL` | Whisper model id (default `whisper-1`) |
| `LLM_MODEL` | Default LLM id for classification-style calls |
| `EMBEDDING_MODEL` | Embeddings model id |

### Dev scripts

| Script | Use |
|--------|-----|
| `scripts/run_migrations.sh` | Apply all `migrations/*.sql` |
| `scripts/generate_token.sh` | Call device auth and pretty-print JSON |
| `scripts/test_voice.sh` | Multipart upload to `/api/v1/voice/upload` |

Make them executable once: `chmod +x scripts/*.sh`.

## MVP roadmap (4 weeks, brief)

- **Week 1 — Foundation:** Axum API, Postgres schema, user + task CRUD, Android shell + Room, Whisper + classifier; deliverable: voice → task in the list (plus text fallback).
- **Week 2 — Widget + loop:** Completion and streaks, Redis-backed widget payload, Glance widget, behavioral signals, journal/thought path; deliverable: home-screen wins and richer voice intents.
- **Week 3 — Profiling + insights:** Question bank, drip scheduling, answer/skip APIs, weekly insight job + UI; deliverable: contextual questions and first weekly insight.
- **Week 4 — Polish + beta:** Qdrant context, offline/sync hardening, rate limits and security pass, accessibility and UI polish, internal Play track; deliverable: closed beta (roughly 10–20 users).

Full day-by-day tables live in **Section 7** of [`../ADHD_APP_ARCHITECTURE.md`](../ADHD_APP_ARCHITECTURE.md).

## Contributing

1. Open an issue or discuss larger changes before heavy implementation.
2. Keep commits focused; match existing Rust and Kotlin style.
3. Run `cargo fmt`, `cargo clippy`, and `cargo test` in `backend/` before submitting.
4. Document new public HTTP behavior in `docs/API.md`.

## License

MIT License — see [LICENSE](LICENSE) (placeholder: add a `LICENSE` file with the standard MIT text when you publish the repo).
