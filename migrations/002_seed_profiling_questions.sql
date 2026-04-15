-- FocusFlow profiling seed: 30 questions (3 per category), stable IDs for depends_on.
-- Run after 001_initial_schema.sql

INSERT INTO profiling_questions (id, category, question_text, question_type, options, priority, depends_on, skip_if, tags, active) VALUES
-- time_perception (10–14)
(
    'ff100000-0001-4000-8000-000000000001',
    'time_perception',
    'Do you often lose track of time without noticing until something (or someone) snaps you out of it?',
    'yes_no',
    NULL,
    10,
    NULL,
    NULL,
    ARRAY['time_perception', 'time_blindness'],
    true
),
(
    'ff100000-0001-4000-8000-000000000002',
    'time_perception',
    'When time slips away from you, what is usually happening?',
    'single_choice',
    '[
        {"id": "hyperfocus", "label": "I am hyperfocused on something interesting"},
        {"id": "avoidance", "label": "I am avoiding a task I do not want to start"},
        {"id": "transitions", "label": "I got sidetracked during a transition (notifications, tabs, rabbit holes)"},
        {"id": "waiting", "label": "I thought I had more time than I did"},
        {"id": "other", "label": "Something else / hard to pin down"}
    ]'::jsonb,
    11,
    'ff100000-0001-4000-8000-000000000001',
    '{"if_parent_answer": "no"}'::jsonb,
    ARRAY['time_perception', 'context'],
    true
),
(
    'ff100000-0001-4000-8000-000000000003',
    'time_perception',
    'In your own words, how does time pressure or lateness show up on your hardest days?',
    'free_text',
    NULL,
    13,
    NULL,
    NULL,
    ARRAY['time_perception', 'narrative'],
    true
),

-- task_initiation (20–24)
(
    'ff100000-0001-4000-8000-000000000004',
    'task_initiation',
    'Starting a new task—even one you genuinely want to do—often feels heavier or stickier than it sounds to others.',
    'yes_no',
    NULL,
    20,
    NULL,
    NULL,
    ARRAY['task_initiation', 'activation'],
    true
),
(
    'ff100000-0001-4000-8000-000000000005',
    'task_initiation',
    'When you feel stuck at the starting line, what tends to be the closest match?',
    'single_choice',
    '[
        {"id": "unclear_first_step", "label": "The first step feels unclear or too big"},
        {"id": "perfectionism", "label": "I want the conditions to be right before I begin"},
        {"id": "boredom", "label": "It feels boring until the last minute"},
        {"id": "anxiety", "label": "Anxiety spikes and I freeze or switch tasks"},
        {"id": "decision_fatigue", "label": "Too many options—hard to commit to one path"}
    ]'::jsonb,
    21,
    'ff100000-0001-4000-8000-000000000004',
    '{"if_parent_answer": "no"}'::jsonb,
    ARRAY['task_initiation', 'barriers'],
    true
),
(
    'ff100000-0001-4000-8000-000000000006',
    'task_initiation',
    'How often does “I will do it in five minutes” turn into an hour or more?',
    'scale_1_5',
    '{"1": "Almost never", "2": "Rarely", "3": "Sometimes", "4": "Often", "5": "Almost always"}'::jsonb,
    23,
    NULL,
    NULL,
    ARRAY['task_initiation', 'procrastination'],
    true
),

-- emotional_regulation (30–34)
(
    'ff100000-0001-4000-8000-000000000007',
    'emotional_regulation',
    'When plans change suddenly or you get interrupted, how intense does the emotional spike feel in the moment? (1 = mild, 5 = very intense)',
    'scale_1_5',
    '{"1": "Mild", "2": "Noticeable", "3": "Moderate", "4": "Strong", "5": "Very intense"}'::jsonb,
    30,
    NULL,
    NULL,
    ARRAY['emotional_regulation', 'reactivity'],
    true
),
(
    'ff100000-0001-4000-8000-000000000008',
    'emotional_regulation',
    'When that spike hits, what do you usually notice first in yourself?',
    'single_choice',
    '[
        {"id": "irritation", "label": "Irritation or anger comes up fast"},
        {"id": "shame", "label": "Shame or self-criticism"},
        {"id": "anxiety", "label": "Anxiety or racing thoughts"},
        {"id": "shutdown", "label": "I go quiet or shut down"},
        {"id": "tears", "label": "Tears or a sudden wave of sadness"}
    ]'::jsonb,
    31,
    'ff100000-0001-4000-8000-000000000007',
    '{"if_parent_answer_lt": "4"}'::jsonb,
    ARRAY['emotional_regulation', 'signals'],
    true
),
(
    'ff100000-0001-4000-8000-000000000009',
    'emotional_regulation',
    'What helps you cool down or reset after overwhelm—even if it does not work every time?',
    'free_text',
    NULL,
    33,
    NULL,
    NULL,
    ARRAY['emotional_regulation', 'recovery'],
    true
),

-- working_memory (40–44)
(
    'ff100000-0001-4000-8000-00000000000a',
    'working_memory',
    'Do you often walk into another room for something specific—and forget what it was by the time you arrive?',
    'yes_no',
    NULL,
    40,
    NULL,
    NULL,
    ARRAY['working_memory', 'doorway_effect'],
    true
),
(
    'ff100000-0001-4000-8000-00000000000b',
    'working_memory',
    'How often does that kind of “walk-in amnesia” happen in a typical week?',
    'scale_1_5',
    '{"1": "Rarely", "2": "Once or twice", "3": "A few times", "4": "Most days", "5": "Multiple times a day"}'::jsonb,
    41,
    'ff100000-0001-4000-8000-00000000000a',
    '{"if_parent_answer": "no"}'::jsonb,
    ARRAY['working_memory', 'frequency'],
    true
),
(
    'ff100000-0001-4000-8000-00000000000c',
    'working_memory',
    'When you try to keep several steps in your head without writing them down, what tends to happen first?',
    'single_choice',
    '[
        {"id": "lose_middle", "label": "I lose the middle steps"},
        {"id": "reorder", "label": "Steps shuffle out of order"},
        {"id": "blank", "label": "I go blank under pressure"},
        {"id": "externalize", "label": "I immediately reach for notes or voice memos"},
        {"id": "hyperfocus_detail", "label": "I over-focus on one step and lose the big picture"}
    ]'::jsonb,
    43,
    NULL,
    NULL,
    ARRAY['working_memory', 'planning'],
    true
),

-- hyperfocus (50–54)
(
    'ff100000-0001-4000-8000-00000000000d',
    'hyperfocus',
    'Do you sometimes slip into such deep focus that hours disappear without you noticing basics like eating, drinking, or stretching?',
    'yes_no',
    NULL,
    50,
    NULL,
    NULL,
    ARRAY['hyperfocus', 'tunnel'],
    true
),
(
    'ff100000-0001-4000-8000-00000000000e',
    'hyperfocus',
    'When that happens, how disruptive is the aftermath (catching up, sleep, relationships, or your body)?',
    'scale_1_5',
    '{"1": "Barely disruptive", "2": "Minor cleanup", "3": "Moderate cost", "4": "Pretty disruptive", "5": "Major fallout"}'::jsonb,
    51,
    'ff100000-0001-4000-8000-00000000000d',
    '{"if_parent_answer": "no"}'::jsonb,
    ARRAY['hyperfocus', 'aftermath'],
    true
),
(
    'ff100000-0001-4000-8000-00000000000f',
    'hyperfocus',
    'What kinds of activities most reliably pull you into that tunnel? (games, research, chats, creative work, etc.)',
    'free_text',
    NULL,
    53,
    NULL,
    NULL,
    ARRAY['hyperfocus', 'triggers'],
    true
),

-- social_impact (60–64)
(
    'ff100000-0001-4000-8000-000000000010',
    'social_impact',
    'How much does ADHD-related inconsistency (lateness, forgotten messages, missed details) affect relationships you care about? (1 = almost none, 5 = a lot)',
    'scale_1_5',
    '{"1": "Almost none", "2": "A little", "3": "Moderate", "4": "Quite a bit", "5": "A lot"}'::jsonb,
    60,
    NULL,
    NULL,
    ARRAY['social_impact', 'relationships'],
    true
),
(
    'ff100000-0001-4000-8000-000000000011',
    'social_impact',
    'Do people sometimes misread your forgetfulness or lateness as not caring—even when you do care?',
    'yes_no',
    NULL,
    61,
    NULL,
    NULL,
    ARRAY['social_impact', 'misunderstanding'],
    true
),
(
    'ff100000-0001-4000-8000-000000000012',
    'social_impact',
    'Name one social situation you would like tools to navigate more smoothly (work, family, dating, group chats, etc.).',
    'free_text',
    NULL,
    63,
    NULL,
    NULL,
    ARRAY['social_impact', 'goals'],
    true
),

-- sleep_patterns (70–74)
(
    'ff100000-0001-4000-8000-000000000013',
    'sleep_patterns',
    'Overall in the past month, how rested do you feel when you wake up? (1 = not rested, 5 = very rested)',
    'scale_1_5',
    '{"1": "Not rested", "2": "Mostly tired", "3": "Okay", "4": "Fairly rested", "5": "Very rested"}'::jsonb,
    70,
    NULL,
    NULL,
    ARRAY['sleep_patterns', 'rest'],
    true
),
(
    'ff100000-0001-4000-8000-000000000014',
    'sleep_patterns',
    'How would you describe your usual bedtime drift?',
    'single_choice',
    '[
        {"id": "consistent", "label": "Fairly consistent"},
        {"id": "revenge_bedtime", "label": "I push bedtime later to reclaim “me time”"},
        {"id": "cannot_wind_down", "label": "I cannot wind down until my brain finally crashes"},
        {"id": "variable", "label": "Highly variable week to week"},
        {"id": "early_wake", "label": "I fall asleep late but wake too early"}
    ]'::jsonb,
    72,
    NULL,
    NULL,
    ARRAY['sleep_patterns', 'rhythm'],
    true
),
(
    'ff100000-0001-4000-8000-000000000015',
    'sleep_patterns',
    'When sleep feels rough, what pattern fits best?',
    'single_choice',
    '[
        {"id": "racing_thoughts", "label": "Racing thoughts at night"},
        {"id": "delayed_sleep", "label": "Delayed sleep phase—I am wired late"},
        {"id": "fragmented", "label": "Fragmented sleep / frequent waking"},
        {"id": "screens", "label": "Hard to stop scrolling or media"},
        {"id": "meds_timing", "label": "Medication timing affects evenings"}
    ]'::jsonb,
    73,
    'ff100000-0001-4000-8000-000000000013',
    '{"if_parent_answer_gt": "3"}'::jsonb,
    ARRAY['sleep_patterns', 'troubleshooting'],
    true
),

-- coping_strategies (80–84)
(
    'ff100000-0001-4000-8000-000000000016',
    'coping_strategies',
    'When overwhelm hits, what do you reach for first?',
    'single_choice',
    '[
        {"id": "break", "label": "A break, walk, or movement"},
        {"id": "music", "label": "Music or podcasts"},
        {"id": "snack", "label": "Food or caffeine"},
        {"id": "scroll", "label": "Scrolling or light distraction"},
        {"id": "talk", "label": "Talking it out with someone"},
        {"id": "power_through", "label": "Trying to power through anyway"}
    ]'::jsonb,
    80,
    NULL,
    NULL,
    ARRAY['coping_strategies', 'habits'],
    true
),
(
    'ff100000-0001-4000-8000-000000000017',
    'coping_strategies',
    'A coping habit you are a little embarrassed by—but it sometimes works. What is it?',
    'free_text',
    NULL,
    81,
    NULL,
    NULL,
    ARRAY['coping_strategies', 'honesty'],
    true
),
(
    'ff100000-0001-4000-8000-000000000018',
    'coping_strategies',
    'How confident are you that your coping toolkit actually gets you unstuck—not just distracted? (1 = not confident, 5 = very confident)',
    'scale_1_5',
    '{"1": "Not confident", "2": "Low", "3": "Mixed", "4": "Mostly confident", "5": "Very confident"}'::jsonb,
    82,
    NULL,
    NULL,
    ARRAY['coping_strategies', 'effectiveness'],
    true
),

-- motivation_drivers (90–94)
(
    'ff100000-0001-4000-8000-000000000019',
    'motivation_drivers',
    'What usually ignites motivation for you?',
    'single_choice',
    '[
        {"id": "deadline", "label": "A real deadline or consequence"},
        {"id": "novelty", "label": "Novelty or learning something new"},
        {"id": "accountability", "label": "Accountability to another person"},
        {"id": "interest", "label": "Pure interest—if it clicks, I fly"},
        {"id": "stakes", "label": "High stakes for someone I care about"},
        {"id": "body_doubling", "label": "Body doubling or parallel work sessions"}
    ]'::jsonb,
    90,
    NULL,
    NULL,
    ARRAY['motivation_drivers', 'spark'],
    true
),
(
    'ff100000-0001-4000-8000-00000000001a',
    'motivation_drivers',
    'Describe a project type you can hyperfocus on even when other priorities exist.',
    'free_text',
    NULL,
    91,
    NULL,
    NULL,
    ARRAY['motivation_drivers', 'interest_patterns'],
    true
),
(
    'ff100000-0001-4000-8000-00000000001b',
    'motivation_drivers',
    'How true is this for you: “I need a deadline breathing down my neck to really move.”',
    'scale_1_5',
    '{"1": "Not true", "2": "Rarely", "3": "Sometimes", "4": "Often", "5": "Very true"}'::jsonb,
    92,
    NULL,
    NULL,
    ARRAY['motivation_drivers', 'pressure'],
    true
),

-- medication (100–104) — asked later; follow-ups only if user indicates medication use
(
    'ff100000-0001-4000-8000-00000000001c',
    'medication',
    'Are you currently taking any prescribed medication for ADHD (or closely related treatment) under the guidance of a clinician?',
    'yes_no',
    NULL,
    100,
    NULL,
    NULL,
    ARRAY['medication', 'clinical'],
    true
),
(
    'ff100000-0001-4000-8000-00000000001d',
    'medication',
    'How consistent has your dosing or routine been lately?',
    'scale_1_5',
    '{"1": "Very inconsistent", "2": "Often miss or shift timing", "3": "Mixed", "4": "Mostly consistent", "5": "Very consistent"}'::jsonb,
    101,
    'ff100000-0001-4000-8000-00000000001c',
    '{"if_parent_answer": "no"}'::jsonb,
    ARRAY['medication', 'adherence'],
    true
),
(
    'ff100000-0001-4000-8000-00000000001e',
    'medication',
    'Which timing pattern feels closest to your medication days?',
    'single_choice',
    '[
        {"id": "morning_only", "label": "Morning only"},
        {"id": "split_dose", "label": "Split dose (morning + afternoon)"},
        {"id": "afternoon_sensitive", "label": "Afternoon dose affects sleep—hard to balance"},
        {"id": "not_daily", "label": "Not daily / as-needed with clinician guidance"},
        {"id": "other", "label": "Other / prefer not to detail"}
    ]'::jsonb,
    102,
    'ff100000-0001-4000-8000-00000000001c',
    '{"if_parent_answer": "no"}'::jsonb,
    ARRAY['medication', 'timing'],
    true
);
