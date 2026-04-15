package com.focusflow.ui.screens

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.focusflow.data.local.FocusFlowDatabase
import com.focusflow.data.local.entity.TaskEntity
import com.focusflow.data.remote.ApiClient
import com.focusflow.data.remote.CreateTaskRequest
import com.focusflow.data.remote.UpdateTaskRequest
import com.focusflow.domain.models.Task
import com.focusflow.domain.models.TaskPriority
import com.focusflow.domain.models.TaskSource
import com.focusflow.domain.models.TaskStatus
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.UUID

data class TasksUiState(
    val completedTasks: List<Task> = emptyList(),
    val pendingTasks: List<Task> = emptyList(),
    val isLoading: Boolean = false,
)

class TasksViewModel(application: Application) : AndroidViewModel(application) {

    private val db = FocusFlowDatabase.getInstance(application)
    private val taskDao = db.taskDao()
    private val api = ApiClient.api

    private val _uiState = MutableStateFlow(TasksUiState())
    val uiState: StateFlow<TasksUiState> = _uiState.asStateFlow()

    init {
        observeTasks()
    }

    fun completeTask(taskId: String) {
        viewModelScope.launch {
            val now = System.currentTimeMillis()
            val pending = _uiState.value.pendingTasks
            val task = pending.find { it.id == taskId } ?: return@launch

            val entity = TaskEntity(
                id = task.id,
                userId = task.userId,
                title = task.title,
                description = task.description,
                status = "completed",
                priority = task.priority.value,
                source = task.source.value,
                dueAt = task.dueAt,
                completedAt = now,
                createdAt = task.createdAt,
                synced = false,
            )
            taskDao.update(entity)

            try {
                api.updateTask(taskId, UpdateTaskRequest(status = "completed"))
                taskDao.update(entity.copy(synced = true))
            } catch (e: Exception) {
                Log.w(TAG, "Failed to sync task completion", e)
            }
        }
    }

    fun createTask(title: String, priority: TaskPriority = TaskPriority.MEDIUM) {
        viewModelScope.launch {
            val localId = UUID.randomUUID().toString()
            val entity = TaskEntity(
                id = localId,
                userId = "",
                title = title,
                priority = priority.value,
                source = "manual",
                synced = false,
            )
            taskDao.insert(entity)

            try {
                val dto = api.createTask(
                    CreateTaskRequest(
                        title = title,
                        priority = priority.value,
                        source = "manual",
                    )
                )
                taskDao.insert(
                    entity.copy(
                        id = dto.id,
                        userId = dto.userId,
                        synced = true,
                    )
                )
            } catch (e: Exception) {
                Log.w(TAG, "Failed to sync new task", e)
            }
        }
    }

    private fun observeTasks() {
        viewModelScope.launch {
            taskDao.getAll().collect { entities ->
                val tasks = entities.map { it.toDomain() }
                _uiState.update { state ->
                    state.copy(
                        completedTasks = tasks.filter { it.status == TaskStatus.COMPLETED },
                        pendingTasks = tasks.filter {
                            it.status == TaskStatus.PENDING || it.status == TaskStatus.IN_PROGRESS
                        },
                    )
                }
            }
        }
    }

    private fun TaskEntity.toDomain() = Task(
        id = id,
        userId = userId,
        title = title,
        description = description,
        status = TaskStatus.fromValue(status),
        priority = TaskPriority.fromValue(priority),
        source = TaskSource.fromValue(source),
        dueAt = dueAt,
        completedAt = completedAt,
        createdAt = createdAt,
    )

    companion object {
        private const val TAG = "TasksViewModel"
    }
}
