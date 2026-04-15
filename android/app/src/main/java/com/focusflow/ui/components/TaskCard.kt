package com.focusflow.ui.components

import android.view.HapticFeedbackConstants
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.focusflow.domain.models.Task
import com.focusflow.domain.models.TaskPriority
import com.focusflow.domain.models.TaskSource
import com.focusflow.ui.theme.Green400
import com.focusflow.ui.theme.PriorityHigh
import com.focusflow.ui.theme.PriorityLow
import com.focusflow.ui.theme.PriorityMedium
import com.focusflow.ui.theme.PriorityUrgent
import com.focusflow.ui.theme.White

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TaskCard(
    task: Task,
    onComplete: () -> Unit,
    onClick: () -> Unit,
    swipeEnabled: Boolean = true,
    modifier: Modifier = Modifier,
) {
    val view = LocalView.current
    var completed by remember { mutableStateOf(false) }

    val bgColor by animateColorAsState(
        targetValue = if (completed) Green400.copy(alpha = 0.15f) else MaterialTheme.colorScheme.surface,
        animationSpec = tween(300),
        label = "taskCardBg",
    )

    if (swipeEnabled) {
        val dismissState = rememberSwipeToDismissBoxState(
            confirmValueChange = { value ->
                if (value == SwipeToDismissBoxValue.StartToEnd) {
                    completed = true
                    view.performHapticFeedback(HapticFeedbackConstants.CONFIRM)
                    onComplete()
                    true
                } else false
            }
        )

        SwipeToDismissBox(
            state = dismissState,
            backgroundContent = {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clip(RoundedCornerShape(12.dp))
                        .background(Green400),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    Icon(
                        imageVector = Icons.Filled.Check,
                        contentDescription = "Complete",
                        tint = White,
                        modifier = Modifier.padding(start = 20.dp),
                    )
                }
            },
            enableDismissFromEndToStart = false,
            modifier = modifier,
        ) {
            TaskCardContent(
                task = task,
                backgroundColor = bgColor,
                onClick = onClick,
            )
        }
    } else {
        TaskCardContent(
            task = task,
            backgroundColor = bgColor,
            onClick = onClick,
            modifier = modifier,
        )
    }
}

@Composable
private fun TaskCardContent(
    task: Task,
    backgroundColor: androidx.compose.ui.graphics.Color,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var expanded by remember { mutableStateOf(false) }

    Card(
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(containerColor = backgroundColor),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
        modifier = modifier
            .fillMaxWidth()
            .clickable {
                expanded = !expanded
                onClick()
            },
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Box(
                    modifier = Modifier
                        .size(10.dp)
                        .clip(CircleShape)
                        .background(task.priority.dotColor()),
                )

                Spacer(modifier = Modifier.width(12.dp))

                Text(
                    text = task.title,
                    style = MaterialTheme.typography.bodyLarge,
                    maxLines = if (expanded) Int.MAX_VALUE else 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f),
                )

                if (task.source == TaskSource.VOICE) {
                    Spacer(modifier = Modifier.width(8.dp))
                    Icon(
                        imageVector = Icons.Filled.Phone,
                        contentDescription = "Voice input",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }

            if (expanded && task.description != null) {
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = task.description,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            if (task.dueAt != null) {
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = formatDueDate(task.dueAt),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            if (task.completedAt != null) {
                Spacer(modifier = Modifier.height(4.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        imageVector = Icons.Filled.Check,
                        contentDescription = null,
                        tint = Green400,
                        modifier = Modifier.size(14.dp),
                    )
                    Spacer(modifier = Modifier.width(4.dp))
                    Text(
                        text = formatCompletionTime(task.completedAt),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

private fun TaskPriority.dotColor() = when (this) {
    TaskPriority.LOW -> PriorityLow
    TaskPriority.MEDIUM -> PriorityMedium
    TaskPriority.HIGH -> PriorityHigh
    TaskPriority.URGENT -> PriorityUrgent
}

private fun formatDueDate(epochMs: Long): String {
    val now = System.currentTimeMillis()
    val diff = epochMs - now
    val hours = diff / (1000 * 60 * 60)
    val days = hours / 24

    return when {
        diff < 0 -> "Overdue"
        hours < 1 -> "Due soon"
        hours < 24 -> "In ${hours}h"
        days == 1L -> "Tomorrow"
        days < 7 -> "In ${days} days"
        else -> {
            val sdf = java.text.SimpleDateFormat("MMM d", java.util.Locale.getDefault())
            sdf.format(java.util.Date(epochMs))
        }
    }
}

private fun formatCompletionTime(epochMs: Long): String {
    val sdf = java.text.SimpleDateFormat("h:mm a", java.util.Locale.getDefault())
    return sdf.format(java.util.Date(epochMs))
}
