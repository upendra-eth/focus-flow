package com.focusflow.ui.screens

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Image
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import coil.compose.AsyncImage

// ---------------------------------------------------------------------------
// Data types used by the UI
// ---------------------------------------------------------------------------

data class ChatMessage(
    val id: String,
    val text: String,
    val isUser: Boolean,
    val entries: List<EntryChipData> = emptyList(),
    val imageUris: List<Uri> = emptyList(),
)

data class EntryChipData(
    val category: String,
    val title: String,
)

// ---------------------------------------------------------------------------
// Main screen
// ---------------------------------------------------------------------------

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AgentChatScreen(viewModel: AgentChatViewModel = viewModel()) {
    val uiState by viewModel.uiState.collectAsState()
    val listState = rememberLazyListState()

    val photoPickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.PickMultipleVisualMedia(maxItems = 4),
    ) { uris: List<Uri> ->
        uris.forEach { viewModel.attachImage(it) }
    }

    LaunchedEffect(uiState.messages.size) {
        if (uiState.messages.isNotEmpty()) {
            listState.animateScrollToItem(uiState.messages.lastIndex)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = uiState.conversationTitle ?: "New Chat",
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.surface,
                ),
            )
        },
        bottomBar = {
            ChatInputBar(
                inputText = uiState.inputText,
                isLoading = uiState.isLoading,
                attachedImages = uiState.attachedImages,
                onTextChange = viewModel::onInputTextChange,
                onSend = viewModel::sendMessage,
                onAttachImage = {
                    photoPickerLauncher.launch(
                        PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly)
                    )
                },
                onRemoveImage = viewModel::removeImage,
                onMicClick = { /* TODO: wire voice recording */ },
            )
        },
    ) { padding ->
        if (uiState.messages.isEmpty() && !uiState.isLoading) {
            EmptyState(modifier = Modifier.padding(padding))
        } else {
            LazyColumn(
                state = listState,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding),
                contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                items(uiState.messages, key = { it.id }) { message ->
                    ChatBubble(message = message, isUser = message.isUser)
                }

                if (uiState.isLoading) {
                    item {
                        TypingIndicator()
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------

@Composable
private fun EmptyState(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text("👋", fontSize = 48.sp)
            Spacer(Modifier.height(12.dp))
            Text(
                text = "Hey! Tell me about your day.",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                text = "I'll organize everything for you.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.outline,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Chat bubble
// ---------------------------------------------------------------------------

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ChatBubble(message: ChatMessage, isUser: Boolean) {
    val screenWidth = LocalConfiguration.current.screenWidthDp.dp
    val alignment = if (isUser) Alignment.CenterEnd else Alignment.CenterStart
    val bgColor = if (isUser) {
        MaterialTheme.colorScheme.primary
    } else {
        MaterialTheme.colorScheme.surfaceVariant
    }
    val textColor = if (isUser) {
        MaterialTheme.colorScheme.onPrimary
    } else {
        MaterialTheme.colorScheme.onSurfaceVariant
    }
    val shape = RoundedCornerShape(
        topStart = 16.dp,
        topEnd = 16.dp,
        bottomStart = if (isUser) 16.dp else 4.dp,
        bottomEnd = if (isUser) 4.dp else 16.dp,
    )

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        contentAlignment = alignment,
    ) {
        Column(
            modifier = Modifier.widthIn(max = screenWidth * 0.8f),
        ) {
            if (message.imageUris.isNotEmpty()) {
                LazyRow(
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                    modifier = Modifier.padding(bottom = 4.dp),
                ) {
                    items(message.imageUris) { uri ->
                        AsyncImage(
                            model = uri,
                            contentDescription = null,
                            modifier = Modifier
                                .size(80.dp)
                                .clip(RoundedCornerShape(8.dp)),
                            contentScale = ContentScale.Crop,
                        )
                    }
                }
            }

            Surface(
                shape = shape,
                color = bgColor,
            ) {
                Text(
                    text = message.text,
                    color = textColor,
                    style = MaterialTheme.typography.bodyLarge,
                    modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp),
                )
            }

            if (message.entries.isNotEmpty()) {
                Spacer(Modifier.height(4.dp))
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    message.entries.forEach { entry ->
                        EntryCategoryChip(category = entry.category, title = entry.title)
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Entry category chip
// ---------------------------------------------------------------------------

@Composable
fun EntryCategoryChip(category: String, title: String) {
    val emoji = when (category) {
        "financial" -> "💰"
        "health" -> "💪"
        "idea" -> "💡"
        "learning" -> "📚"
        "relationship" -> "👥"
        "goal" -> "🎯"
        "gratitude" -> "🙏"
        "day_recap" -> "📋"
        "mind_dump" -> "🧠"
        else -> "📝"
    }

    Surface(
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.tertiaryContainer,
    ) {
        Text(
            text = "$emoji $category: $title",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onTertiaryContainer,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

// ---------------------------------------------------------------------------
// Typing indicator
// ---------------------------------------------------------------------------

@Composable
private fun TypingIndicator() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        contentAlignment = Alignment.CenterStart,
    ) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surfaceVariant,
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                CircularProgressIndicator(
                    modifier = Modifier.size(14.dp),
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.outline,
                )
                Spacer(Modifier.width(6.dp))
                Text(
                    text = "Thinking…",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.outline,
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Chat input bar
// ---------------------------------------------------------------------------

@Composable
private fun ChatInputBar(
    inputText: String,
    isLoading: Boolean,
    attachedImages: List<Uri>,
    onTextChange: (String) -> Unit,
    onSend: () -> Unit,
    onAttachImage: () -> Unit,
    onRemoveImage: (Int) -> Unit,
    onMicClick: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.surface),
    ) {
        AnimatedVisibility(
            visible = attachedImages.isNotEmpty(),
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            AttachmentPreview(
                images = attachedImages,
                onRemove = onRemoveImage,
            )
        }

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 8.dp, vertical = 8.dp),
            verticalAlignment = Alignment.Bottom,
        ) {
            IconButton(onClick = onAttachImage) {
                Icon(
                    Icons.Default.Image,
                    contentDescription = "Attach image",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            TextField(
                value = inputText,
                onValueChange = onTextChange,
                modifier = Modifier.weight(1f),
                placeholder = { Text("What's on your mind?") },
                maxLines = 4,
                shape = RoundedCornerShape(24.dp),
                colors = TextFieldDefaults.colors(
                    focusedIndicatorColor = Color.Transparent,
                    unfocusedIndicatorColor = Color.Transparent,
                    focusedContainerColor = MaterialTheme.colorScheme.surfaceVariant,
                    unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant,
                ),
            )

            if (inputText.isBlank() && attachedImages.isEmpty()) {
                IconButton(onClick = onMicClick) {
                    Icon(
                        Icons.Default.Mic,
                        contentDescription = "Voice input",
                        tint = MaterialTheme.colorScheme.primary,
                    )
                }
            } else {
                IconButton(
                    onClick = onSend,
                    enabled = !isLoading && inputText.isNotBlank(),
                ) {
                    Icon(
                        Icons.AutoMirrored.Filled.Send,
                        contentDescription = "Send",
                        tint = if (!isLoading && inputText.isNotBlank()) {
                            MaterialTheme.colorScheme.primary
                        } else {
                            MaterialTheme.colorScheme.outline
                        },
                    )
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Attachment preview row
// ---------------------------------------------------------------------------

@Composable
fun AttachmentPreview(images: List<Uri>, onRemove: (Int) -> Unit) {
    LazyRow(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        items(images.size) { index ->
            Box(modifier = Modifier.size(72.dp)) {
                AsyncImage(
                    model = images[index],
                    contentDescription = "Attached image",
                    modifier = Modifier
                        .matchParentSize()
                        .clip(RoundedCornerShape(8.dp)),
                    contentScale = ContentScale.Crop,
                )
                Surface(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .size(20.dp)
                        .clickable { onRemove(index) },
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.error,
                ) {
                    Icon(
                        Icons.Default.Close,
                        contentDescription = "Remove",
                        tint = MaterialTheme.colorScheme.onError,
                        modifier = Modifier
                            .padding(2.dp)
                            .size(16.dp),
                    )
                }
            }
        }
    }
}
