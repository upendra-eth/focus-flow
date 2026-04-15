package com.focusflow.data.local.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "widget_state")
data class WidgetStateEntity(
    @PrimaryKey
    val id: Int = 1,

    @ColumnInfo(name = "completed_today")
    val completedToday: Int = 0,

    @ColumnInfo(name = "streak_days")
    val streakDays: Int = 0,

    @ColumnInfo(name = "motivational_message")
    val motivationalMessage: String = "Ready when you are.",

    @ColumnInfo(name = "last_completed_task_title")
    val lastCompletedTaskTitle: String? = null,

    @ColumnInfo(name = "last_updated")
    val lastUpdated: Long = System.currentTimeMillis()
)
