# FocusFlow — Practical Deployment Guide

## Table of Contents
1. [What Runs Where — Phone vs Server](#1-what-runs-where)
2. [Required External APIs & Costs](#2-required-apis--costs)
3. [Step-by-Step: Get APK on Your Phone](#3-get-apk-on-your-phone)
4. [Step-by-Step: Run Backend](#4-run-backend)
5. [DevOps for Light/Personal Use](#5-devops-for-light-use)
6. [Cheapest Production Setup](#6-cheapest-production-setup)

---

## 1. What Runs Where

### What MUST run on a server (cannot be on-phone)

| Component | Why it needs a server | Can it move to phone later? |
|---|---|---|
| **PostgreSQL** | Primary data store, multi-device sync | Only if you go single-device with SQLite (Room already does this) |
| **OpenAI Whisper API** | Transcribes voice to text | YES — use Android's built-in `SpeechRecognizer` instead (free, on-device, no API key) |
| **LLM (GPT-4o-mini)** | Intent classification | PARTIALLY — use on-device regex/rules for simple cases, LLM only for ambiguous |
| **LLM (GPT-4o)** | Weekly insights | NO — needs large model, but runs once/week so cost is minimal |

### What already runs on the phone (no server needed)

| Component | Status | Notes |
|---|---|---|
| **Room Database** | Already built | Full offline task CRUD, widget state |
| **Glance Widget** | Already built | Reads from local DataStore |
| **Voice Recording** | Already built | MediaRecorder captures audio |
| **UI / Navigation** | Already built | Full Jetpack Compose app |
| **Sync Engine** | Already built | WorkManager, syncs when online |
| **Behavioral Signals** | Can be local-only | Store in Room, send to server in batch |

### Simplified Architecture for Personal Use

```
YOUR PHONE (offline-capable)
├── Voice input → Android SpeechRecognizer (FREE, on-device)
├── Simple intent detection → local regex rules
├── Task storage → Room SQLite
├── Widget → Glance (reads local DB)
├── Profiling questions → bundled in APK
└── Sync to server when online ↓

LIGHTWEIGHT SERVER (only needed for AI features)
├── API server (Rust) → tiny VPS
├── PostgreSQL → same VPS or managed free tier
├── GPT-4o-mini → intent classification for ambiguous inputs
├── GPT-4o → weekly insights (1 call/week)
└── Redis → optional, skip for personal use
```

### What you can SKIP entirely for personal use

| Component | Skip? | Why |
|---|---|---|
| **Redis** | YES | Widget cache is overkill for 1 user. Room DB is fast enough. |
| **Qdrant (vector DB)** | YES | Semantic search matters at scale. For 1 user, simple SQL queries work fine. |
| **NATS (event bus)** | YES | Event-driven architecture is for microservices. Single binary doesn't need it. |
| **Whisper API** | YES | Use Android's free `SpeechRecognizer` instead. Saves ~$10/mo. |
| **Embeddings API** | YES | Skip vector search, use keyword matching for question selection. |
| **WebSocket** | YES | Pull-based sync via WorkManager is fine for 1 user. |

**Bottom line: For personal use, you need the Rust API + PostgreSQL + 1 OpenAI API key. That's it.**

---

## 2. Required APIs & Costs

### Must-Have API

| API | What for | Cost (personal use) | Free alternative |
|---|---|---|---|
| **OpenAI API** (GPT-4o-mini) | Classify voice input into task/journal/answer | ~$0.50/month (assuming ~100 voice inputs/month at $0.15/1M input tokens) | On-device rules for simple inputs, LLM only for ambiguous |
| **OpenAI API** (GPT-4o) | Weekly insights | ~$0.20/month (4 calls/month) | Skip insights initially |

**Total API cost: ~$0.70/month for personal use.**

### Optional APIs

| API | What for | Cost | Skip if... |
|---|---|---|---|
| OpenAI Whisper | Speech-to-text | ~$3/month | Use Android SpeechRecognizer (free) |
| OpenAI Embeddings | Semantic question selection | ~$0.10/month | Use priority-based selection instead |
| Anthropic Claude | Better insights | ~$0.30/month | GPT-4o is fine |

### Recommended: Use Android On-Device Speech (FREE)

Replace Whisper API with Android's built-in speech recognition:
- Works offline on most modern Android phones
- Zero cost, zero latency for network
- Accuracy is good enough for task capture
- Falls back to server-side only if on-device fails

---

## 3. Get APK on Your Phone

### Prerequisites
1. Open **Docker Desktop** app (click the whale icon in Applications)
2. Open **Android Studio** (already installed at `/Applications/Android Studio.app`)
3. Enable **Developer Options** on your Android phone:
   - Settings → About Phone → tap "Build Number" 7 times
   - Settings → Developer Options → enable "USB Debugging"
4. Connect phone via USB cable

### Option A: Build & Install via Android Studio (recommended)

```bash
# 1. Open Android Studio
open "/Applications/Android Studio.app"

# 2. Open the project: File → Open → navigate to:
#    /Users/upendrasingh/data/ai-assistance/focusflow/android

# 3. Wait for Gradle sync to complete (first time takes 5-10 min)

# 4. Connect your phone via USB
#    - Accept "Allow USB debugging?" prompt on phone

# 5. Select your phone from the device dropdown (top toolbar)

# 6. Click the green Run ▶ button
#    - This builds the APK and installs it directly on your phone
```

### Option B: Build APK from command line

```bash
# 1. Set ANDROID_HOME
export ANDROID_HOME=~/Library/Android/sdk
export PATH=$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools:$PATH

# 2. Build debug APK
cd /Users/upendrasingh/data/ai-assistance/focusflow/android
./gradlew assembleDebug

# 3. APK will be at:
#    android/app/build/outputs/apk/debug/app-debug.apk

# 4. Install on connected phone
adb install app/build/outputs/apk/debug/app-debug.apk
```

### Option C: Just transfer the APK file

```bash
# After building, copy APK to phone
adb push app/build/outputs/apk/debug/app-debug.apk /sdcard/Download/

# On phone: open Files app → Downloads → tap FocusFlow APK → Install
# (You'll need to allow "Install from unknown sources" in settings)
```

### Update API URL for your phone

Before building, update the API URL in the Android build config:

**For testing on same WiFi network:**
```
# Find your Mac's local IP
ifconfig en0 | grep inet

# Then update android/app/build.gradle.kts:
# Change: buildConfigField("String", "API_BASE_URL", "\"http://10.0.2.2:8080\"")
# To:     buildConfigField("String", "API_BASE_URL", "\"http://YOUR_MAC_IP:8080\"")
```

**For production (deployed server):**
```
buildConfigField("String", "API_BASE_URL", "\"https://your-server.com\"")
```

---

## 4. Run Backend

### Step 1: Start Docker Desktop

Click Docker Desktop in Applications, wait for the whale icon to show "running".

### Step 2: Start infrastructure

```bash
cd /Users/upendrasingh/data/ai-assistance/focusflow

# Start PostgreSQL, Redis, Qdrant, NATS
docker compose up -d

# Verify everything is running
docker compose ps
```

### Step 3: Configure API keys

```bash
cp .env.example .env

# Edit .env — the ONLY required key is:
# OPENAI_API_KEY=sk-your-actual-key-here
#
# Get one at: https://platform.openai.com/api-keys
# Add $5 credit to start (will last months for personal use)
```

### Step 4: Run database migrations

```bash
# Wait for Postgres to be ready
sleep 3

# Run migrations
psql postgres://focusflow:focusflow_dev@localhost:5432/focusflow \
  -f migrations/001_initial_schema.sql

psql postgres://focusflow:focusflow_dev@localhost:5432/focusflow \
  -f migrations/002_seed_profiling_questions.sql
```

### Step 5: Start the API server

```bash
cd backend
cargo run -p focusflow-api
# Server starts on http://localhost:8080
```

### Step 6: Test it works

```bash
# In a new terminal:
# Register a device
curl -s -X POST http://localhost:8080/api/v1/auth/device \
  -H "Content-Type: application/json" \
  -d '{"device_id": "my-phone-001"}' | python3 -m json.tool

# Should return: { "token": "eyJ...", "user": { ... } }
```

---

## 5. DevOps for Light/Personal Use

### Best Model: "Laptop Server" (Zero Cost)

For personal use / "sometimes" usage, **don't deploy to the cloud at all**.

```
How it works:
1. Run Docker + backend on your Mac when you want AI features
2. Phone connects to Mac over local WiFi
3. When Mac is off, phone works fully offline (tasks, widget, everything)
4. When Mac comes back online, phone syncs automatically

Cost: $0/month (just your OpenAI API key at ~$0.70/mo)
```

**This is the recommended approach for "I use it sometimes".**

### If You Want It Always-On: Cheapest Cloud Options

| Provider | Spec | Cost | Best for |
|---|---|---|---|
| **Oracle Cloud Free Tier** | 4 ARM CPUs, 24GB RAM, 200GB disk | **$0/month FOREVER** | Best free option. Runs Rust + Postgres easily. |
| **Hetzner CAX11** | 2 ARM vCPU, 4GB RAM, 40GB | **€3.79/month (~$4)** | Cheapest paid option in EU. |
| **Railway.app** | Pay-per-use | **~$5/month** at light usage | Easiest deploy (git push). Free $5 trial. |
| **Fly.io** | 1 shared CPU, 256MB | **~$3/month** | Good for Rust (small binary). Free tier available. |
| **DigitalOcean Droplet** | 1 vCPU, 512MB | **$4/month** | Simple, reliable. |

### Recommended: Oracle Cloud Free Tier

```
Why:
- Actually free (not trial — permanently free tier)
- 4 ARM CPUs + 24GB RAM is MORE than enough
- Can run Postgres + Rust API + Redis all on one machine
- 200GB disk
- Located worldwide (choose region closest to you)

Setup time: ~30 minutes
```

### Deploy Steps (Oracle Cloud or any VPS)

```bash
# 1. SSH into your server
ssh ubuntu@your-server-ip

# 2. Install Docker
curl -fsSL https://get.docker.com | sh

# 3. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. Clone your project
git clone <your-repo> focusflow
cd focusflow

# 5. Start everything
docker compose up -d
cd backend && cargo build --release
./target/release/focusflow-api &

# 6. Set up HTTPS with Caddy (free SSL)
# Install Caddy, configure reverse proxy to :8080
```

### Recommended CI/CD: Keep It Simple

For personal use, skip CI/CD pipelines entirely. Just:

```bash
# On your VPS, when you want to update:
git pull
cargo build --release
systemctl restart focusflow  # if using systemd
```

If you want automation later:
- **GitHub Actions** (free for public repos, 2000 min/mo for private)
- Single workflow: build Rust binary → Docker image → deploy to VPS via SSH

---

## 6. Summary: What You Actually Need

### Minimum Viable Setup (personal use)

```
COST BREAKDOWN:
├── Phone: Free (you already have it)
├── Backend server: Free (run on Mac, or Oracle Cloud free tier)
├── PostgreSQL: Free (Docker on same server)
├── OpenAI API: ~$0.70/month
├── Redis: SKIP (not needed for 1 user)
├── Qdrant: SKIP (not needed for 1 user)
├── NATS: SKIP (not needed for 1 user)
├── Domain name: Optional ($10/year if you want HTTPS)
└── TOTAL: ~$0.70/month
```

### What to do RIGHT NOW

```
Step 1: Open Docker Desktop
Step 2: Open terminal, run: cd focusflow && docker compose up -d
Step 3: Set up .env with your OpenAI key
Step 4: Run migrations (psql commands above)
Step 5: cargo run -p focusflow-api
Step 6: Open Android Studio → open android/ folder → Run on phone
Step 7: Done. Use the app.
```

### Development Model Recommendation

| Phase | What | DevOps |
|---|---|---|
| **Now** | Run backend on Mac, build APK locally | Zero infra. Just Docker + Android Studio. |
| **When it works well** | Deploy to Oracle Cloud free VPS | SSH + git pull to update. 30min setup. |
| **If others want to use it** | Add GitHub Actions CI, proper domain + HTTPS | Still < $5/month total. |
| **If 100+ users** | Move to Hetzner/Fly.io, add Redis, proper monitoring | ~$15-30/month. |
