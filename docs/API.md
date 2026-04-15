# FocusFlow HTTP & WebSocket API

Base URL for local development: **`http://localhost:8080/api/v1`**

All paths below are relative to that base unless noted.

---

## Authentication

### Device registration / login

There is no separate signup flow for the MVP. The client sends a stable **device ID**; the server creates a user row on first sight or loads an existing one and returns a **JWT**.

- **HTTP (except `/auth/device` and WebSocket upgrade):** send  
  `Authorization: Bearer <access_token>`
- **WebSocket:** pass the same JWT as a query parameter (see [WebSocket](#websocket)).

JWTs are issued by `POST /auth/device` and validated on each protected request. Typical failure: **`401 Unauthorized`** with a JSON body `{ "error": "...", "details": ... }`.

---

## Error format

Most errors return JSON:

```json
{
  "error": "short machine-oriented message",
  "details": "optional longer string"
}
```

Common status codes:

| Code | When |
|------|------|
| `400` | Bad JSON, missing multipart field, invalid id |
| `401` | Missing `Authorization`, invalid or expired JWT, bad WS token |
| `404` | Resource not found (where applicable) |
| `500` | Internal errors (database, AI provider, etc.) |

---

## Endpoints

### 1. `POST /auth/device`

Register or log in with a device identifier.

**Request**

```json
{
  "device_id": "dev-test-device-001"
}
```

**Response** `200 OK`

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "device_id": "dev-test-device-001",
    "email": null,
    "display_name": null,
    "timezone": "UTC",
    "created_at": "2026-04-14T12:00:00Z",
    "last_active_at": "2026-04-14T12:00:00Z",
    "onboarding_done": false,
    "settings": {}
  }
}
```

**Errors**

- `500` — `failed to create user`, `database error`, or `failed to generate token`

---

### 2. `POST /voice/upload`

Upload a short audio clip for transcription and intent classification. **Content-Type:** `multipart/form-data` with a field named **`audio`** (file bytes).

**Headers**

```
Authorization: Bearer <token>
```

**Response** `200 OK`

```json
{
  "transcript": "Remind me to call the pharmacy tomorrow afternoon",
  "intent": "create_task",
  "confidence": 0.92,
  "action_taken": "created_task",
  "data": {
    "title": "Call the pharmacy",
    "due_hint": "tomorrow afternoon",
    "priority_hint": "medium"
  }
}
```

`intent` is one of: `create_task`, `thought_dump`, `profile_answer`, `unclear` (snake_case in JSON). The `data` object shape depends on `intent` (task fields, thought summary, profile match, or raw text).

**Errors**

- `400` — `invalid multipart data`, `missing 'audio' field in multipart form`, read errors
- `401` — missing/invalid bearer token
- `500` — `transcription failed`, classification/persistence failures

---

### 3. `GET /tasks`

List the authenticated user’s tasks.

**Query parameters**

| Name | Description |
|------|-------------|
| `status` | Optional filter: e.g. `pending`, `in_progress`, `completed`, `abandoned`, or `all` |

**Response** `200 OK` — JSON array of task objects:

```json
[
  {
    "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Buy groceries",
    "description": null,
    "status": "pending",
    "priority": "medium",
    "source": "manual",
    "due_at": null,
    "completed_at": null,
    "created_at": "2026-04-14T10:00:00Z",
    "tags": [],
    "ai_metadata": null
  }
]
```

**Errors**

- `401` — auth failure
- `500` — `failed to list tasks`

---

### 4. `POST /tasks`

Create a task manually (not from voice).

**Request**

```json
{
  "title": "Write weekly summary",
  "description": "Include metrics from the dashboard",
  "priority": "high",
  "due_at": "2026-04-18T17:00:00Z",
  "tags": ["work", "writing"]
}
```

`description`, `priority` (defaults to `medium`), `due_at`, and `tags` are optional.

**Response** `201 Created` — single task object (same shape as in the list).

**Errors**

- `401` — auth failure
- `500` — `failed to create task`

---

### 5. `PATCH /tasks/{id}`

Update a task. `{id}` is a UUID path segment (Axum route: `/api/v1/tasks/{id}`).

**Request** (all fields optional; send only what changes)

```json
{
  "status": "completed",
  "title": "Updated title",
  "description": "New description",
  "priority": "low",
  "due_at": "2026-04-20T09:00:00Z"
}
```

Setting `"status": "completed"` runs completion logic (including timestamps) when supported.

**Response** `200 OK` — updated task object.

**Errors**

- `400` / `404` — invalid id or not found (implementation-dependent)
- `401` — auth failure
- `500` — `failed to update task`

---

### 6. `GET /widget/state`

Return cached/widget-oriented stats for the dopamine home widget.

**Response** `200 OK`

```json
{
  "completed_today": 3,
  "streak_days": 5,
  "motivational_message": "You're on a roll — three wins today.",
  "last_completed_task_title": "Inbox zero",
  "last_updated": "2026-04-14T15:30:00Z"
}
```

**Errors**

- `401` — auth failure
- `500` — `failed to get widget state`

---

### 7. `GET /profile/next-question`

Return the next profiling question for this user, or `null` if none is due.

**Response** `200 OK` — question object or `null`:

```json
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "category": "task_initiation",
  "question_text": "When you think of starting a task, what happens first?",
  "question_type": "single_choice",
  "options": ["I freeze", "I distract myself", "I jump in immediately"],
  "priority": 10,
  "depends_on": null,
  "skip_if": null,
  "tags": ["adhd", "initiation"],
  "active": true
}
```

**Errors**

- `401` — auth failure
- `500` — `failed to select question`

---

### 8. `POST /profile/answer`

Submit an answer to a profiling question.

**Request**

```json
{
  "question_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "answer_value": "I freeze",
  "raw_input": "I usually freeze for like ten minutes"
}
```

`raw_input` is optional (e.g. original voice text).

**Response** `200 OK` — stored answer record:

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "question_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "answer_value": "I freeze",
  "raw_input": "I usually freeze for like ten minutes",
  "source": "manual",
  "confidence": 1.0,
  "answered_at": "2026-04-14T16:00:00Z",
  "session_context": null
}
```

**Errors**

- `401` — auth failure
- `500` — `failed to submit answer`

---

### 9. `POST /profile/skip`

Record that the user skipped a question (so scheduling can move on).

**Request**

```json
{
  "question_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
}
```

**Response** `200 OK`

```json
{
  "success": true
}
```

**Errors**

- `401` — auth failure
- `500` — `failed to skip question`

---

### 10. `GET /insights/latest`

Return the most recent weekly insight for the user.

**Response** `200 OK` — insight object or `null` if none exists yet:

```json
{
  "id": "11111111-2222-3333-4444-555555555555",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "week_start": "2026-04-07",
  "summary_text": "You completed more tasks in the morning than the evening.",
  "patterns": { "morning_completions": 8, "evening_completions": 2 },
  "recommendations": { "suggestions": ["Try scheduling hard tasks before noon"] },
  "tasks_completed": 12,
  "streak_days": 4,
  "generated_at": "2026-04-14T08:00:00Z"
}
```

**Errors**

- `401` — auth failure
- `500` — `failed to get insight`

---

### 11. `POST /signals`

Batch-ingest **behavioral signals** (app open, task events, etc.).

**Request**

```json
{
  "signals": [
    {
      "signal_type": "app_foreground",
      "payload": { "session_id": "abc123" }
    },
    {
      "signal_type": "task_completed",
      "payload": { "task_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8" }
    }
  ]
}
```

**Response** `200 OK`

```json
{
  "recorded": 2
}
```

Individual signal failures may be skipped with a warning in server logs; `recorded` is the count of successful writes.

**Errors**

- `401` — auth failure
- `500` — unexpected failures in the handler path

---

## WebSocket

### Connect

**URL:** `ws://localhost:8080/api/v1/ws?token=<JWT>`

The JWT is the same string returned by `POST /auth/device`. Browsers and mobile clients use the WebSocket upgrade on this path; **no** `Authorization` header is used for the upgrade—only the **`token`** query parameter.

**Upgrade errors** (JSON body, no WebSocket):

| Status | `error` |
|--------|---------|
| `401` | `invalid or expired token` |
| `400` | `invalid user id in token` |

### Server messages

Messages are **JSON text** frames. They use an envelope with a `type` discriminator and optional `payload`:

```json
{ "type": "Ping" }
```

```json
{
  "type": "TaskUpdate",
  "payload": { "...": "task object fields" }
}
```

```json
{
  "type": "WidgetUpdate",
  "payload": {
    "completed_today": 3,
    "streak_days": 5,
    "motivational_message": "Nice streak!",
    "last_completed_task_title": "Walk the dog",
    "last_updated": "2026-04-14T15:30:00Z"
  }
}
```

```json
{
  "type": "QuestionReady",
  "payload": { "...": "profiling question fields" }
}
```

Variant names match Rust enums: `TaskUpdate`, `WidgetUpdate`, `QuestionReady`, `Ping`.

### Client messages

The server may log incoming text frames; there is no required client protocol beyond responding to WebSocket **Ping** with **Pong** (handled by many libraries). The server periodically sends JSON **`Ping`** messages over the text channel for keep-alive.

### TLS / production

In production, use `wss://` with a valid TLS certificate and the same path and query layout.

---

## Summary table

| # | Method | Path | Auth |
|---|--------|------|------|
| 1 | `POST` | `/auth/device` | No |
| 2 | `POST` | `/voice/upload` | Bearer |
| 3 | `GET` | `/tasks` | Bearer |
| 4 | `POST` | `/tasks` | Bearer |
| 5 | `PATCH` | `/tasks/{id}` | Bearer |
| 6 | `GET` | `/widget/state` | Bearer |
| 7 | `GET` | `/profile/next-question` | Bearer |
| 8 | `POST` | `/profile/answer` | Bearer |
| 9 | `POST` | `/profile/skip` | Bearer |
| 10 | `GET` | `/insights/latest` | Bearer |
| 11 | `POST` | `/signals` | Bearer |
| — | `WS` | `/ws?token=...` | Query token |
