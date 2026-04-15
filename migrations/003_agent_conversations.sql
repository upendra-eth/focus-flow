-- ============================================================
-- CONVERSATIONS (a thread of user ↔ agent messages)
-- ============================================================
CREATE TABLE conversations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_conversations_user ON conversations(user_id, updated_at DESC);

-- ============================================================
-- CONVERSATION MESSAGES (each turn in a conversation)
-- ============================================================
CREATE TABLE conversation_messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL,       -- 'user' or 'assistant'
    content         TEXT NOT NULL,
    image_urls      TEXT[],              -- optional attached image URLs
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_messages_conv ON conversation_messages(conversation_id, created_at);

-- ============================================================
-- LIFE ENTRIES (categorized from agent conversations)
-- The agent auto-creates these when it detects structured info
-- ============================================================
CREATE TABLE life_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category        TEXT NOT NULL,
    -- Categories:
    --   'daily_note'      — general thoughts, how the day went
    --   'mind_dump'       — unstructured brain dump, venting
    --   'day_recap'       — structured summary of what user did today
    --   'financial'       — expenses, income, money-related
    --   'health'          — exercise, sleep, medication, food
    --   'idea'            — creative ideas, project concepts
    --   'learning'        — things learned, articles read, insights
    --   'relationship'    — social interactions, people mentioned
    --   'goal'            — goals, aspirations, plans
    --   'gratitude'       — things user is grateful for
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    structured_data JSONB,               -- category-specific structured fields
    -- e.g. financial: {"amount": 45.00, "type": "expense", "category": "food"}
    -- e.g. health:    {"type": "exercise", "duration_min": 30, "activity": "walk"}
    -- e.g. day_recap: {"mood": "good", "energy": 7, "highlights": [...]}
    source_message_id UUID REFERENCES conversation_messages(id),
    image_urls      TEXT[],
    tags            TEXT[] NOT NULL DEFAULT '{}',
    entry_date      DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_life_entries_user_date ON life_entries(user_id, entry_date DESC);
CREATE INDEX idx_life_entries_user_cat  ON life_entries(user_id, category);

-- ============================================================
-- DAILY DASHBOARDS (AI-generated daily summary)
-- ============================================================
CREATE TABLE daily_dashboards (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dashboard_date  DATE NOT NULL,
    summary_text    TEXT NOT NULL,
    mood_score      INT,                  -- 1-10 inferred from entries
    energy_score    INT,                  -- 1-10 inferred from entries
    categories_breakdown JSONB NOT NULL,  -- {"financial": 2, "daily_note": 5, ...}
    highlights      JSONB NOT NULL,       -- [{"text": "...", "category": "..."}]
    financial_summary JSONB,              -- {"total_spent": 120, "total_earned": 0, "items": [...]}
    generated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, dashboard_date)
);

-- ============================================================
-- IMAGE UPLOADS (metadata for user-attached images)
-- ============================================================
CREATE TABLE image_uploads (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename        TEXT NOT NULL,
    mime_type       TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL,
    storage_path    TEXT NOT NULL,         -- local path or cloud URL
    ai_description  TEXT,                  -- Gemini-generated description
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_images_user ON image_uploads(user_id, created_at DESC);
