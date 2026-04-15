package com.focusflow.ui.screens

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.focusflow.data.local.FocusFlowDatabase
import com.focusflow.data.local.entity.TaskEntity
import com.focusflow.data.remote.ApiClient
import com.focusflow.data.remote.CreateTaskRequest
import com.focusflow.data.remote.SkipQuestionRequest
import com.focusflow.data.remote.SubmitAnswerRequest
import com.focusflow.domain.models.ProfilingQuestion
import com.focusflow.domain.models.Task
import com.focusflow.domain.models.TaskPriority
import com.focusflow.domain.models.TaskSource
import com.focusflow.domain.models.TaskStatus
import com.focusflow.domain.models.VoiceResult
import com.focusflow.ui.components.ConfirmationMessage
import com.focusflow.ui.components.RecordingState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.UUID

data class HomeUiState(
    val inputText: String = "",
    val recordingState: RecordingState = RecordingState.IDLE,
    val lastConfirmation: ConfirmationMessage? = null,
    val showConfirmation: Boolean = false,
    val currentQuestion: ProfilingQuestion? = null,
    val isLoading: Boolean = false,
    val lastCompletedTask: Task? = null,
)

class HomeViewModel(application: Application) : AndroidViewModel(application) {

    private val db = FocusFlowDatabase.getInstance(application)
    private val taskDao = db.taskDao()
    private val api = ApiClient.api

    private val _uiState = MutableStateFlow(HomeUiState())
    val uiState: StateFlow<HomeUiState> = _uiState.asStateFlow()

    init {
        loadNextQuestion()
        loadLastCompletedTask()
    }

    fun onInputChanged(text: String) {
        _uiState.update { it.copy(inputText = text) }
    }

    fun onTextSubmit() {
        val text = _uiState.value.inputText.trim()
        if (text.isBlank()) return

        _uiState.update { it.copy(isLoading = true, inputText = "") }

        viewModelScope.launch {
            try {
                val taskDto = api.createTask(
                    CreateTaskRequest(
                        title = text,
                        priority = "medium",
                        source = "manual",
                    )
                )
                val entity = TaskEntity(
                    id = taskDto.id,
                    userId = taskDto.userId,
                    title = taskDto.title,
                    description = taskDto.description,
                    status = taskDto.status,
                    priority = taskDto.priority,
                    source = taskDto.source,
                    createdAt = System.currentTimeMillis(),
                    synced = true,
                )
                taskDao.insert(entity)

                _uiState.update {
                    it.copy(
                        isLoading = false,
                        lastConfirmation = ConfirmationMessage.TaskAdded(taskDto.title),
                        showConfirmation = true,
                    )
                }
            } catch (e: Exception) {
                Log.w(TAG, "API create failed, saving locally", e)
                val localId = UUID.randomUUID().toString()
                taskDao.insert(
                    TaskEntity(
                        id = localId,
                        userId = "",
                        title = text,
                        synced = false,
                    )
                )
                _uiState.update {
                    it.copy(
                        isLoading = false,
                        lastConfirmation = ConfirmationMessage.TaskAdded(text),
                        showConfirmation = true,
                    )
                }
            }
        }
    }

    fun onVoiceRecorded(audioBytes: ByteArray) {
        _uiState.update { it.copy(recordingState = RecordingState.PROCESSING) }

        viewModelScope.launch {
            try {
                val requestBody = audioBytes.toRequestBody("audio/mp4".toMediaTypeOrNull())
                val part = MultipartBody.Part.createFormData("audio", "recording.m4a", requestBody)
                val response = api.uploadVoice(part)

                val confirmation = if (response.task != null) {
                    val taskDto = response.task
                    taskDao.insert(
                        TaskEntity(
                            id = taskDto.id,
                            userId = taskDto.userId,
                            title = taskDto.title,
                            description = taskDto.description,
                            status = taskDto.status,
                            priority = taskDto.priority,
                            source = "voice",
                            createdAt = System.currentTimeMillis(),
                            synced = true,
                        )
                    )
                    ConfirmationMessage.TaskAdded(taskDto.title)
                } else {
                    ConfirmationMessage.ThoughtNoted
                }

                _uiState.update {
                    it.copy(
                        recordingState = RecordingState.IDLE,
                        lastConfirmation = confirmation,
                        showConfirmation = true,
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "Voice upload failed", e)
                _uiState.update { it.copy(recordingState = RecordingState.IDLE) }
            }
        }
    }

    fun onRecordingStarted() {
        _uiState.update { it.copy(recordingState = RecordingState.RECORDING) }
    }

    fun onRecordingStopped() {
        // Actual byte processing is handled via onVoiceRecorded
    }

    fun onQuestionAnswered(answer: String) {
        val question = _uiState.value.currentQuestion ?: return

        viewModelScope.launch {
            try {
                api.submitAnswer(
                    SubmitAnswerRequest(
                        questionId = question.id,
                        answerValue = answer,
                    )
                )
            } catch (e: Exception) {
                Log.w(TAG, "Failed to submit answer", e)
            }
            _uiState.update { it.copy(currentQuestion = null) }
            loadNextQuestion()
        }
    }

    fun onQuestionSkipped() {
        val question = _uiState.value.currentQuestion ?: return

        viewModelScope.launch {
            try {
                api.skipQuestion(SkipQuestionRequest(questionId = question.id))
            } catch (e: Exception) {
                Log.w(TAG, "Failed to skip question", e)
            }
            _uiState.update { it.copy(currentQuestion = null) }
            loadNextQuestion()
        }
    }

    fun dismissConfirmation() {
        _uiState.update { it.copy(showConfirmation = false) }
    }

    private fun loadNextQuestion() {
        viewModelScope.launch {
            try {
                val dto = api.getNextQuestion()
                val question = dto?.let {
                    ProfilingQuestion(
                        id = it.id,
                        category = it.category,
                        questionText = it.questionText,
                        questionType = it.questionType,
                        options = it.options,
                        tags = it.tags,
                    )
                }
                _uiState.update { it.copy(currentQuestion = question) }
            } catch (e: Exception) {
                Log.w(TAG, "Failed to load profiling question", e)
            }
        }
    }

    private fun loadLastCompletedTask() {
        viewModelScope.launch {
            val startOfDay = java.util.Calendar.getInstance().apply {
                set(java.util.Calendar.HOUR_OF_DAY, 0)
                set(java.util.Calendar.MINUTE, 0)
                set(java.util.Calendar.SECOND, 0)
                set(java.util.Calendar.MILLISECOND, 0)
            }.timeInMillis

            taskDao.getCompletedToday(startOfDay).collect { entities ->
                val latest = entities.firstOrNull()
                _uiState.update { state ->
                    state.copy(
                        lastCompletedTask = latest?.let {
                            Task(
                                id = it.id,
                                userId = it.userId,
                                title = it.title,
                                description = it.description,
                                status = TaskStatus.COMPLETED,
                                priority = TaskPriority.fromValue(it.priority),
                                source = TaskSource.fromValue(it.source),
                                completedAt = it.completedAt,
                                createdAt = it.createdAt,
                            )
                        }
                    )
                }
            }
        }
    }

    companion object {
        private const val TAG = "HomeViewModel"
    }
}
