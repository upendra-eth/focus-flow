package com.focusflow.data.local.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "tasks")
data class TaskEntity(
    @PrimaryKey
    val id: String,

    @ColumnInfo(name = "user_id")
    val userId: String,

    val title: String,

    val description: String? = null,

    val status: String = "pending",

    val priority: String = "medium",

    val source: String = "manual",

    @ColumnInfo(name = "due_at")
    val dueAt: Long? = null,

    @ColumnInfo(name = "completed_at")
    val completedAt: Long? = null,

    @ColumnInfo(name = "created_at")
    val createdAt: Long = System.currentTimeMillis(),

    val tags: String = "[]",

    @ColumnInfo(name = "ai_metadata")
    val aiMetadata: String? = null,

    val synced: Boolean = false
)
