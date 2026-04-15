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

// --- Agent ---

data class AgentChatRequest(
    @SerializedName("conversation_id") val conversation_id: String? = null,
    val text: String,
    val images: List<ImageAttachmentDto>? = null,
)

data class ImageAttachmentDto(
    @SerializedName("mime_type") val mime_type: String,
    val data: String,
)

data class AgentChatResponse(
    @SerializedName("conversation_id") val conversation_id: String,
    val reply: String,
    @SerializedName("entries_created") val entries_created: List<EntryBriefDto>,
    @SerializedName("web_search_used") val web_search_used: Boolean,
)

data class EntryBriefDto(
    val id: String,
    val category: String,
    val title: String,
)

data class LifeEntryDto(
    val id: String,
    val category: String,
    val title: String,
    val content: String,
    @SerializedName("structured_data") val structuredData: Map<String, Any>? = null,
    val tags: List<String> = emptyList(),
    @SerializedName("entry_date") val entryDate: String,
    @SerializedName("created_at") val createdAt: String,
)

data class DashboardDto(
    @SerializedName("summary_text") val summary_text: String,
    @SerializedName("mood_score") val mood_score: Int? = null,
    @SerializedName("energy_score") val energy_score: Int? = null,
    @SerializedName("categories_breakdown") val categories_breakdown: Map<String, Int> = emptyMap(),
    val highlights: List<Map<String, String>> = emptyList(),
    @SerializedName("financial_summary") val financial_summary: Map<String, Any>? = null,
)

data class ConversationDto(
    val id: String,
    val title: String? = null,
    @SerializedName("updated_at") val updatedAt: String,
)
