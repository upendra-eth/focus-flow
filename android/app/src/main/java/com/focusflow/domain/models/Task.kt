package com.focusflow.domain.models

data class Task(
    val id: String,
    val userId: String,
    val title: String,
    val description: String? = null,
    val status: TaskStatus = TaskStatus.PENDING,
    val priority: TaskPriority = TaskPriority.MEDIUM,
    val source: TaskSource = TaskSource.MANUAL,
    val dueAt: Long? = null,
    val completedAt: Long? = null,
    val createdAt: Long = System.currentTimeMillis(),
    val tags: List<String> = emptyList(),
    val aiMetadata: Map<String, Any>? = null
)

enum class TaskStatus(val value: String) {
    PENDING("pending"),
    IN_PROGRESS("in_progress"),
    COMPLETED("completed"),
    ABANDONED("abandoned");

    companion object {
        fun fromValue(value: String): TaskStatus =
            entries.firstOrNull { it.value == value } ?: PENDING
    }
}

enum class TaskPriority(val value: String) {
    LOW("low"),
    MEDIUM("medium"),
    HIGH("high"),
    URGENT("urgent");

    companion object {
        fun fromValue(value: String): TaskPriority =
            entries.firstOrNull { it.value == value } ?: MEDIUM
    }
}

enum class TaskSource(val value: String) {
    MANUAL("manual"),
    VOICE("voice"),
    AI_SUGGESTED("ai_suggested");

    companion object {
        fun fromValue(value: String): TaskSource =
            entries.firstOrNull { it.value == value } ?: MANUAL
    }
}

data class WidgetState(
    val completedToday: Int = 0,
    val streakDays: Int = 0,
    val motivationalMessage: String = "Ready when you are.",
    val lastCompletedTaskTitle: String? = null,
    val lastUpdated: Long = System.currentTimeMillis()
)

data class ProfilingQuestion(
    val id: String,
    val category: String,
    val questionText: String,
    val questionType: String,
    val options: List<String>? = null,
    val tags: List<String> = emptyList()
)

data class WeeklyInsight(
    val id: String,
    val weekStart: String,
    val summaryText: String,
    val patterns: Map<String, Double> = emptyMap(),
    val recommendations: List<Recommendation> = emptyList(),
    val tasksCompleted: Int = 0,
    val streakDays: Int = 0,
    val generatedAt: String
)

data class Recommendation(
    val action: String,
    val reason: String
)

data class VoiceResult(
    val intent: String,
    val confidence: Double,
    val transcript: String,
    val task: Task? = null,
    val thoughtId: String? = null
)
