package com.focusflow.data.remote

import com.google.gson.annotations.SerializedName

// --- Auth ---

data class DeviceAuthRequest(
    @SerializedName("device_id") val deviceId: String
)

data class AuthResponse(
    @SerializedName("user_id") val userId: String,
    val token: String
)

// --- Voice ---

data class VoiceResponse(
    val intent: String,
    val confidence: Double,
    val transcript: String,
    val task: TaskDto? = null,
    @SerializedName("thought_id") val thoughtId: String? = null
)

// --- Tasks ---

data class TaskDto(
    val id: String,
    @SerializedName("user_id") val userId: String,
    val title: String,
    val description: String? = null,
    val status: String = "pending",
    val priority: String = "medium",
    val source: String = "manual",
    @SerializedName("due_at") val dueAt: String? = null,
    @SerializedName("completed_at") val completedAt: String? = null,
    @SerializedName("created_at") val createdAt: String,
    val tags: List<String> = emptyList(),
    @SerializedName("ai_metadata") val aiMetadata: Map<String, Any>? = null
)

data class CreateTaskRequest(
    val title: String,
    val description: String? = null,
    val priority: String = "medium",
    val source: String = "manual",
    @SerializedName("due_at") val dueAt: String? = null,
    val tags: List<String> = emptyList()
)

data class UpdateTaskRequest(
    val title: String? = null,
    val description: String? = null,
    val status: String? = null,
    val priority: String? = null,
    @SerializedName("due_at") val dueAt: String? = null,
    val tags: List<String>? = null
)

// --- Widget ---

data class WidgetStateDto(
    @SerializedName("completed_today") val completedToday: Int,
    @SerializedName("streak_days") val streakDays: Int,
    @SerializedName("motivational_message") val motivationalMessage: String,
    @SerializedName("last_completed_task_title") val lastCompletedTaskTitle: String? = null,
    @SerializedName("last_updated") val lastUpdated: String
)

// --- Profiling ---

data class QuestionDto(
    val id: String,
    val category: String,
    @SerializedName("question_text") val questionText: String,
    @SerializedName("question_type") val questionType: String,
    val options: List<String>? = null,
    val tags: List<String> = emptyList()
)

data class SubmitAnswerRequest(
    @SerializedName("question_id") val questionId: String,
    @SerializedName("answer_value") val answerValue: String,
    val source: String = "direct_ask"
)

data class AnswerDto(
    val id: String,
    @SerializedName("question_id") val questionId: String,
    @SerializedName("answer_value") val answerValue: String,
    @SerializedName("answered_at") val answeredAt: String
)

data class SkipQuestionRequest(
    @SerializedName("question_id") val questionId: String
)

// --- Insights ---

data class InsightDto(
    val id: String,
    @SerializedName("week_start") val weekStart: String,
    @SerializedName("summary_text") val summaryText: String,
    val patterns: Map<String, Double> = emptyMap(),
    val recommendations: List<RecommendationDto> = emptyList(),
    @SerializedName("tasks_completed") val tasksCompleted: Int,
    @SerializedName("streak_days") val streakDays: Int,
    @SerializedName("generated_at") val generatedAt: String
)

data class RecommendationDto(
    val action: String,
    val reason: String
)

// --- Signals ---

data class SendSignalsRequest(
    val signals: List<SignalDto>
)

data class SignalDto(
    @SerializedName("signal_type") val signalType: String,
    val payload: Map<String, Any> = emptyMap(),
    @SerializedName("recorded_at") val recordedAt: String? = null
)
