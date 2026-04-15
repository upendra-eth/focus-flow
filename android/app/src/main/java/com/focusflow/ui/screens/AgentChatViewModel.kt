package com.focusflow.ui.screens

import android.app.Application
import android.net.Uri
import android.util.Base64
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.focusflow.data.remote.AgentChatRequest
import com.focusflow.data.remote.ApiClient
import com.focusflow.data.remote.ImageAttachmentDto
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.UUID

data class AgentChatUiState(
    val messages: List<ChatMessage> = emptyList(),
    val inputText: String = "",
    val isLoading: Boolean = false,
    val conversationId: String? = null,
    val conversationTitle: String? = null,
    val attachedImages: List<Uri> = emptyList(),
    val error: String? = null,
)

class AgentChatViewModel(application: Application) : AndroidViewModel(application) {

    private val _uiState = MutableStateFlow(AgentChatUiState())
    val uiState: StateFlow<AgentChatUiState> = _uiState.asStateFlow()

    private val api = ApiClient.instance

    fun onInputTextChange(text: String) {
        _uiState.update { it.copy(inputText = text) }
    }

    fun attachImage(uri: Uri) {
        _uiState.update { state ->
            if (state.attachedImages.size >= 4) state
            else state.copy(attachedImages = state.attachedImages + uri)
        }
    }

    fun removeImage(index: Int) {
        _uiState.update { state ->
            state.copy(attachedImages = state.attachedImages.toMutableList().apply {
                removeAt(index)
            })
        }
    }

    fun loadConversation(conversationId: String) {
        _uiState.update { it.copy(conversationId = conversationId) }
    }

    fun sendMessage() {
        val state = _uiState.value
        val text = state.inputText.trim()
        if (text.isBlank()) return

        val userMsg = ChatMessage(
            id = UUID.randomUUID().toString(),
            text = text,
            isUser = true,
            imageUris = state.attachedImages.toList(),
        )

        _uiState.update {
            it.copy(
                messages = it.messages + userMsg,
                inputText = "",
                attachedImages = emptyList(),
                isLoading = true,
                error = null,
            )
        }

        viewModelScope.launch {
            try {
                val imageAttachments = encodeImages(state.attachedImages)

                val request = AgentChatRequest(
                    conversation_id = _uiState.value.conversationId,
                    text = text,
                    images = imageAttachments.ifEmpty { null },
                )

                val response = api.agentChat(request)

                val assistantMsg = ChatMessage(
                    id = UUID.randomUUID().toString(),
                    text = response.reply,
                    isUser = false,
                    entries = response.entries_created.map { e ->
                        EntryChipData(category = e.category, title = e.title)
                    },
                )

                _uiState.update {
                    it.copy(
                        messages = it.messages + assistantMsg,
                        isLoading = false,
                        conversationId = response.conversation_id,
                        conversationTitle = it.conversationTitle ?: text.take(40),
                    )
                }
            } catch (e: Exception) {
                _uiState.update {
                    it.copy(
                        isLoading = false,
                        error = e.message ?: "Something went wrong",
                    )
                }
            }
        }
    }

    private fun encodeImages(uris: List<Uri>): List<ImageAttachmentDto> {
        val context = getApplication<Application>()
        return uris.mapNotNull { uri ->
            try {
                val bytes = context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                    ?: return@mapNotNull null
                val mimeType = context.contentResolver.getType(uri) ?: "image/jpeg"
                val base64 = Base64.encodeToString(bytes, Base64.NO_WRAP)
                ImageAttachmentDto(mime_type = mimeType, data = base64)
            } catch (_: Exception) {
                null
            }
        }
    }
}
