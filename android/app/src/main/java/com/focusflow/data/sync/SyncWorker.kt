package com.focusflow.data.sync

import android.content.Context
import android.util.Log
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import com.focusflow.data.local.FocusFlowDatabase
import com.focusflow.data.local.entity.TaskEntity
import com.focusflow.data.remote.ApiClient
import com.focusflow.data.remote.CreateTaskRequest
import com.focusflow.data.remote.UpdateTaskRequest
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

class SyncWorker(
    context: Context,
    params: WorkerParameters
) : CoroutineWorker(context, params) {

    private val db = FocusFlowDatabase.getInstance(applicationContext)
    private val api = ApiClient.api
    private val gson = Gson()

    override suspend fun doWork(): Result {
        return try {
            pushLocalChanges()
            pullRemoteTasks()
            Result.success()
        } catch (e: Exception) {
            Log.e(TAG, "Sync failed", e)
            Result.retry()
        }
    }

    private suspend fun pushLocalChanges() {
        val unsynced = db.taskDao().getUnsynced()
        for (task in unsynced) {
            try {
                val tags: List<String> = gson.fromJson(
                    task.tags,
                    object : TypeToken<List<String>>() {}.type
                ) ?: emptyList()

                api.createTask(
                    CreateTaskRequest(
                        title = task.title,
                        description = task.description,
                        priority = task.priority,
                        source = task.source,
                        tags = tags
                    )
                )
                db.taskDao().update(task.copy(synced = true))
            } catch (e: Exception) {
                Log.w(TAG, "Failed to push task ${task.id}", e)
            }
        }
    }

    private suspend fun pullRemoteTasks() {
        try {
            val remoteTasks = api.getTasks()
            val entities = remoteTasks.map { dto ->
                TaskEntity(
                    id = dto.id,
                    userId = dto.userId,
                    title = dto.title,
                    description = dto.description,
                    status = dto.status,
                    priority = dto.priority,
                    source = dto.source,
                    dueAt = dto.dueAt?.let { parseIsoTimestamp(it) },
                    completedAt = dto.completedAt?.let { parseIsoTimestamp(it) },
                    createdAt = parseIsoTimestamp(dto.createdAt),
                    tags = gson.toJson(dto.tags),
                    aiMetadata = dto.aiMetadata?.let { gson.toJson(it) },
                    synced = true
                )
            }
            db.taskDao().insertAll(entities)
        } catch (e: Exception) {
            Log.w(TAG, "Failed to pull remote tasks", e)
        }
    }

    private fun parseIsoTimestamp(iso: String): Long {
        return try {
            java.time.Instant.parse(iso).toEpochMilli()
        } catch (_: Exception) {
            System.currentTimeMillis()
        }
    }

    companion object {
        const val TAG = "SyncWorker"
        const val WORK_NAME = "focusflow_sync"
    }
}
