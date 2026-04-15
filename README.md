# FocusFlow — ADHD Productivity + Life Dashboard App

FocusFlow is an AI-powered productivity app designed for ADHD users. It combines **voice-first input**, **dopamine-driven engagement**, and a personal **AI agent** that auto-categorizes your life into a daily dashboard — finances, health, ideas, daily recaps, and more.

## Features

- **AI Agent Chat** — Dump your thoughts naturally. The agent auto-categorizes everything into life entries (daily notes, financial, health, ideas, learning, goals, gratitude, etc.) and builds a daily dashboard.
- **Image + Web Search** — Attach images for the agent to analyze. Ask questions and it searches the web for answers.
- **Daily Dashboard** — AI-generated summary with mood/energy scores, category breakdown, highlights, and financial tracking.
- **Voice-first input** — Ramble freely; the AI classifies intent (task, journal, profile answer, and more).
- **Dopamine home screen widget** — Surfaces wins and streaks, not a guilt list of open to-dos.
- **Drip-feed profiling** — One or two questions at a time, shown in context.
- **Weekly AI insights** — Behavioral patterns plus concrete, actionable recommendations.
- **Offline-first with real-time sync** — Local persistence on device with background sync.

## Tech Stack

| Layer | Choices |
|--------|---------|
| **Backend** | Rust (Axum), PostgreSQL, Redis |
| **AI** | Google Gemini 2.0 Flash (multimodal: text + images + web search grounding) |
| **Mobile** | Android (Jetpack Compose, Glance widget, Room, Coil) |

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust **1.77+** (stable)
- Android Studio **Hedgehog** or newer with **SDK 34**
- Google Gemini API key ([get one free](https://aistudio.google.com/apikey))

### Backend setup

```bash
cd focusflow
cp .env.example .env
# Edit .env — add your GEMINI_API_KEY and JWT_SECRET

# Start infrastructure (Postgres + Redis)
docker compose up -d

# Run SQL migrations
chmod +x scripts/run_migrations.sh
./scripts/run_migrations.sh

# Build and run the API
cd backend
cargo run -p focusflow-api
```

### Android setup

1. Open `android/` in Android Studio.
2. Adjust `API_BASE_URL` in `android/app/build.gradle.kts` to your backend URL.
3. Build and run on emulator or device.

## Project Structure

```
focusflow/
├── backend/                      # Rust workspace
│   ├── api/                      # Axum HTTP + WebSocket server
│   │   └── src/routes/
│   │       ├── agent.rs          # AI agent chat, entries, dashboard
│   │       ├── tasks.rs          # Task CRUD
│   │       ├── voice.rs          # Voice upload + transcription
│   │       ├── profile.rs        # Profiling engine
│   │       └── ...
│   ├── ai/                       # AI service integrations
│   │   └── src/
│   │       ├── agent.rs          # Gemini multimodal agent
│   │       ├── dashboard_generator.rs
│   │       ├── whisper.rs        # Gemini STT
│   │       ├── classifier.rs     # Intent classification
│   │       └── insights_generator.rs
│   ├── core/                     # Domain logic
│   │   └── src/
│   │       ├── agent/            # Agent service orchestration
│   │       ├── tasks/
│   │       ├── profiling/
│   │       └── insights/
│   └── db/                       # Database layer
│
├── android/                      # Android app (Jetpack Compose)
│   └── app/src/main/java/.../
│       ├── ui/screens/
│       │   ├── AgentChatScreen.kt      # Chat with AI agent
│       │   ├── AgentChatViewModel.kt   # Chat state management
│       │   ├── DashboardScreen.kt      # Daily life dashboard
│       │   ├── HomeScreen.kt           # Widget-like home
│       │   └── TasksScreen.kt          # Task management
│       └── data/remote/                # API client + DTOs
│
├── migrations/                   # PostgreSQL migrations (run in order)
│   ├── 001_initial_schema.sql
│   ├── 002_seed_profiling_questions.sql
│   └── 003_agent_conversations.sql
├── scripts/                      # Dev helpers
├── docker-compose.yml
├── .env.example
└── README.md
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/auth/device` | Device-based authentication |
| `POST` | `/api/v1/agent/chat` | Chat with AI agent (text + images) |
| `GET` | `/api/v1/agent/entries` | Get life entries (filterable by date/category) |
| `GET` | `/api/v1/agent/dashboard` | Get daily dashboard |
| `GET` | `/api/v1/agent/conversations` | List conversations |
| `POST` | `/api/v1/voice/upload` | Voice transcription + intent classification |
| `GET/POST` | `/api/v1/tasks` | Task CRUD |
| `PATCH` | `/api/v1/tasks/:id` | Update task |
| `GET` | `/api/v1/widget/state` | Widget data |
| `GET` | `/api/v1/profile/next-question` | Next profiling question |
| `POST` | `/api/v1/profile/answer` | Submit profiling answer |
| `GET` | `/api/v1/insights/latest` | Weekly AI insight |
| `WS` | `/api/v1/ws` | Real-time updates |

## Environment Variables

See `.env.example`:

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis connection URL |
| `GEMINI_API_KEY` | Google Gemini API key (free tier available) |
| `JWT_SECRET` | Secret for signing device JWTs |
| `RUST_LOG` | Tracing filter (e.g. `focusflow_api=debug`) |
| `SERVER_HOST` / `SERVER_PORT` | API bind address |

## How the Agent Works

1. User sends text (+ optional images) via chat
2. Gemini multimodal AI analyzes the input with conversation context
3. Agent auto-categorizes into life entry types:
   - `daily_note` — general thoughts
   - `mind_dump` — unstructured brain dump
   - `day_recap` — what you did today (mood, energy, highlights)
   - `financial` — expenses/income (amount, category extracted)
   - `health` — exercise, sleep, food, medication
   - `idea` — creative ideas, project concepts
   - `learning` — things learned or read
   - `goal` — aspirations, plans
   - `gratitude` — things you're thankful for
4. Structured data is extracted per category (e.g., financial amounts, exercise duration)
5. Daily dashboard is AI-generated from all entries: summary, mood/energy scores, category breakdown, highlights, financial totals

## License

MIT
