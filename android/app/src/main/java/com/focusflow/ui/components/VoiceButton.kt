package com.focusflow.ui.components

import android.Manifest
import android.content.pm.PackageManager
import android.view.HapticFeedbackConstants
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.FloatingActionButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import com.focusflow.ui.theme.Grey400
import com.focusflow.ui.theme.RecordingRed
import com.focusflow.ui.theme.White

enum class RecordingState { IDLE, RECORDING, PROCESSING }

@Composable
fun VoiceButton(
    recordingState: RecordingState,
    onStartRecording: () -> Unit,
    onStopRecording: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val view = LocalView.current
    var showPermissionDialog by remember { mutableStateOf(false) }

    val permissionLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { granted ->
        if (granted) {
            view.performHapticFeedback(HapticFeedbackConstants.LONG_PRESS)
            onStartRecording()
        } else {
            showPermissionDialog = true
        }
    }

    val containerColor by animateColorAsState(
        targetValue = when (recordingState) {
            RecordingState.IDLE -> MaterialTheme.colorScheme.primary
            RecordingState.RECORDING -> RecordingRed
            RecordingState.PROCESSING -> Grey400
        },
        animationSpec = tween(300),
        label = "voiceButtonColor",
    )

    val pulseTransition = rememberInfiniteTransition(label = "pulse")
    val pulseScale by pulseTransition.animateFloat(
        initialValue = 1f,
        targetValue = 1.15f,
        animationSpec = infiniteRepeatable(
            animation = tween(600),
            repeatMode = RepeatMode.Reverse,
        ),
        label = "pulseScale",
    )

    val scale = if (recordingState == RecordingState.RECORDING) pulseScale else 1f

    if (showPermissionDialog) {
        AlertDialog(
            onDismissRequest = { showPermissionDialog = false },
            title = { Text("Microphone Access") },
            text = {
                Text(
                    "To capture your voice, I need mic access. " +
                            "Your audio is processed and deleted — I never store recordings."
                )
            },
            confirmButton = {
                TextButton(onClick = {
                    showPermissionDialog = false
                    permissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                }) { Text("Grant Access") }
            },
            dismissButton = {
                TextButton(onClick = { showPermissionDialog = false }) {
                    Text("Not Now")
                }
            },
        )
    }

    Box(contentAlignment = Alignment.Center, modifier = modifier) {
        FloatingActionButton(
            onClick = {
                when (recordingState) {
                    RecordingState.IDLE -> {
                        val hasMicPermission = ContextCompat.checkSelfPermission(
                            context, Manifest.permission.RECORD_AUDIO
                        ) == PackageManager.PERMISSION_GRANTED

                        if (hasMicPermission) {
                            view.performHapticFeedback(HapticFeedbackConstants.LONG_PRESS)
                            onStartRecording()
                        } else {
                            permissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                        }
                    }
                    RecordingState.RECORDING -> {
                        view.performHapticFeedback(HapticFeedbackConstants.LONG_PRESS)
                        onStopRecording()
                    }
                    RecordingState.PROCESSING -> { /* no-op while processing */ }
                }
            },
            modifier = Modifier
                .size(56.dp)
                .scale(scale),
            shape = CircleShape,
            containerColor = containerColor,
            elevation = FloatingActionButtonDefaults.elevation(
                defaultElevation = 4.dp,
                pressedElevation = 8.dp,
            ),
        ) {
            when (recordingState) {
                RecordingState.PROCESSING -> {
                    CircularProgressIndicator(
                        modifier = Modifier.size(24.dp),
                        color = White,
                        strokeWidth = 2.dp,
                    )
                }
                else -> {
                    Icon(
                        imageVector = Icons.Filled.Phone,
                        contentDescription = when (recordingState) {
                            RecordingState.IDLE -> "Start recording"
                            RecordingState.RECORDING -> "Stop recording"
                            else -> "Processing"
                        },
                        tint = White,
                    )
                }
            }
        }
    }
}
