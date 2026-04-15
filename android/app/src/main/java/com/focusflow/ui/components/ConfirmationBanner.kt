package com.focusflow.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.focusflow.ui.theme.Green100
import com.focusflow.ui.theme.Green500
import kotlinx.coroutines.delay

sealed class ConfirmationMessage {
    data class TaskAdded(val title: String) : ConfirmationMessage()
    data object ThoughtNoted : ConfirmationMessage()
}

@Composable
fun ConfirmationBanner(
    message: ConfirmationMessage?,
    visible: Boolean,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    LaunchedEffect(visible) {
        if (visible) {
            delay(3000)
            onDismiss()
        }
    }

    AnimatedVisibility(
        visible = visible && message != null,
        enter = slideInVertically(
            initialOffsetY = { it },
            animationSpec = tween(300),
        ),
        exit = slideOutVertically(
            targetOffsetY = { it },
            animationSpec = tween(300),
        ),
        modifier = modifier,
    ) {
        Card(
            shape = RoundedCornerShape(12.dp),
            colors = CardDefaults.cardColors(containerColor = Green100),
            modifier = Modifier
                .fillMaxWidth()
                .clickable { onDismiss() },
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
            ) {
                Icon(
                    imageVector = Icons.Filled.Check,
                    contentDescription = null,
                    tint = Green500,
                )
                Spacer(modifier = Modifier.width(12.dp))
                Text(
                    text = when (message) {
                        is ConfirmationMessage.TaskAdded ->
                            "Got it — I added '${message.title}' to your tasks. \u2713"
                        is ConfirmationMessage.ThoughtNoted ->
                            "Noted that down."
                        null -> ""
                    },
                    style = MaterialTheme.typography.bodyMedium,
                    color = Green500,
                )
            }
        }
    }
}
