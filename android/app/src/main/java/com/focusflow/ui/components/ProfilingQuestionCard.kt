package com.focusflow.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.focusflow.domain.models.ProfilingQuestion

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ProfilingQuestionCard(
    question: ProfilingQuestion,
    visible: Boolean,
    onAnswer: (String) -> Unit,
    onSkip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    AnimatedVisibility(
        visible = visible,
        enter = slideInHorizontally(initialOffsetX = { it }),
        exit = slideOutHorizontally(targetOffsetX = { -it }),
        modifier = modifier,
    ) {
        Card(
            shape = RoundedCornerShape(16.dp),
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surfaceVariant,
            ),
            elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(modifier = Modifier.padding(20.dp)) {
                Text(
                    text = question.questionText,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                )

                Spacer(modifier = Modifier.height(16.dp))

                when (question.questionType) {
                    "scale_1_5" -> ScaleInput(onAnswer = onAnswer)
                    "single_choice" -> SingleChoiceInput(
                        options = question.options.orEmpty(),
                        onAnswer = onAnswer,
                    )
                    "yes_no" -> YesNoInput(onAnswer = onAnswer)
                    "free_text" -> FreeTextInput(onAnswer = onAnswer)
                    else -> FreeTextInput(onAnswer = onAnswer)
                }

                Spacer(modifier = Modifier.height(8.dp))

                TextButton(
                    onClick = onSkip,
                    modifier = Modifier.align(Alignment.End),
                ) {
                    Text(
                        text = "Skip",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun ScaleInput(onAnswer: (String) -> Unit) {
    var selected by remember { mutableIntStateOf(-1) }

    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceEvenly,
    ) {
        (1..5).forEach { value ->
            val isSelected = selected == value
            OutlinedButton(
                onClick = {
                    selected = value
                    onAnswer(value.toString())
                },
                colors = ButtonDefaults.outlinedButtonColors(
                    containerColor = if (isSelected)
                        MaterialTheme.colorScheme.primary
                    else
                        MaterialTheme.colorScheme.surface,
                    contentColor = if (isSelected)
                        MaterialTheme.colorScheme.onPrimary
                    else
                        MaterialTheme.colorScheme.onSurface,
                ),
                modifier = Modifier.width(48.dp),
            ) {
                Text(
                    text = value.toString(),
                    textAlign = TextAlign.Center,
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SingleChoiceInput(
    options: List<String>,
    onAnswer: (String) -> Unit,
) {
    var selected by remember { mutableStateOf<String?>(null) }

    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        modifier = Modifier.fillMaxWidth(),
    ) {
        options.forEach { option ->
            FilterChip(
                selected = selected == option,
                onClick = {
                    selected = option
                    onAnswer(option)
                },
                label = { Text(option) },
                colors = FilterChipDefaults.filterChipColors(
                    selectedContainerColor = MaterialTheme.colorScheme.primaryContainer,
                    selectedLabelColor = MaterialTheme.colorScheme.onPrimaryContainer,
                ),
            )
        }
    }
}

@Composable
private fun YesNoInput(onAnswer: (String) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterHorizontally),
    ) {
        Button(
            onClick = { onAnswer("yes") },
            colors = ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.primary,
            ),
        ) {
            Text("Yes")
        }
        OutlinedButton(onClick = { onAnswer("no") }) {
            Text("No")
        }
    }
}

@Composable
private fun FreeTextInput(onAnswer: (String) -> Unit) {
    var text by remember { mutableStateOf("") }

    Column {
        OutlinedTextField(
            value = text,
            onValueChange = { text = it },
            modifier = Modifier.fillMaxWidth(),
            placeholder = { Text("Type your answer...") },
            shape = RoundedCornerShape(12.dp),
            singleLine = false,
            maxLines = 3,
        )
        Spacer(modifier = Modifier.height(8.dp))
        Button(
            onClick = { if (text.isNotBlank()) onAnswer(text.trim()) },
            enabled = text.isNotBlank(),
            modifier = Modifier.align(Alignment.End),
        ) {
            Text("Submit")
        }
    }
}
