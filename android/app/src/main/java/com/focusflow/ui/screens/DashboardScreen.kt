package com.focusflow.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.focusflow.data.remote.ApiClient
import com.focusflow.data.remote.DashboardDto
import kotlinx.coroutines.launch
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.TextStyle
import java.util.Locale

// ---------------------------------------------------------------------------
// Dashboard screen
// ---------------------------------------------------------------------------

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DashboardScreen() {
    var selectedDate by remember { mutableStateOf(LocalDate.now()) }
    var dashboard by remember { mutableStateOf<DashboardDto?>(null) }
    var isLoading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    val api = remember { ApiClient.instance }

    fun loadDashboard(date: LocalDate) {
        scope.launch {
            isLoading = true
            dashboard = try {
                api.getDashboard(date.format(DateTimeFormatter.ISO_LOCAL_DATE))
            } catch (_: Exception) {
                null
            }
            isLoading = false
        }
    }

    LaunchedEffect(selectedDate) {
        loadDashboard(selectedDate)
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Dashboard") },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.surface,
                ),
            )
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 16.dp),
        ) {
            DateSelector(
                date = selectedDate,
                onPrev = { selectedDate = selectedDate.minusDays(1) },
                onNext = {
                    if (selectedDate < LocalDate.now()) {
                        selectedDate = selectedDate.plusDays(1)
                    }
                },
            )

            Spacer(Modifier.height(12.dp))

            if (isLoading) {
                LinearProgressIndicator(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(4.dp)),
                )
            } else if (dashboard == null) {
                EmptyDashboard()
            } else {
                dashboard?.let { d -> DashboardContent(d) }
            }

            Spacer(Modifier.height(24.dp))
        }
    }
}

// ---------------------------------------------------------------------------
// Date selector
// ---------------------------------------------------------------------------

@Composable
private fun DateSelector(
    date: LocalDate,
    onPrev: () -> Unit,
    onNext: () -> Unit,
) {
    val isToday = date == LocalDate.now()
    val dayName = if (isToday) "Today" else date.dayOfWeek.getDisplayName(TextStyle.FULL, Locale.getDefault())
    val formatted = date.format(DateTimeFormatter.ofPattern("MMM d, yyyy"))

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconButton(onClick = onPrev) {
            Icon(Icons.AutoMirrored.Filled.KeyboardArrowLeft, contentDescription = "Previous day")
        }

        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                text = dayName,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = formatted,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.outline,
            )
        }

        IconButton(onClick = onNext, enabled = !isToday) {
            Icon(
                Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = "Next day",
                tint = if (isToday) {
                    MaterialTheme.colorScheme.outline.copy(alpha = 0.3f)
                } else {
                    MaterialTheme.colorScheme.onSurface
                },
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------

@Composable
private fun EmptyDashboard() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 64.dp),
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text("📭", fontSize = 48.sp)
            Spacer(Modifier.height(12.dp))
            Text(
                text = "No entries today",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                text = "Start chatting with your AI assistant\nto build your daily summary.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.outline,
                textAlign = TextAlign.Center,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Dashboard content
// ---------------------------------------------------------------------------

@Composable
private fun DashboardContent(dashboard: DashboardDto) {
    SummaryCard(dashboard.summary_text)

    Spacer(Modifier.height(12.dp))

    MoodEnergyRow(
        mood = dashboard.mood_score,
        energy = dashboard.energy_score,
    )

    Spacer(Modifier.height(12.dp))

    if (dashboard.categories_breakdown.isNotEmpty()) {
        CategoryBreakdown(dashboard.categories_breakdown)
        Spacer(Modifier.height(12.dp))
    }

    if (dashboard.highlights.isNotEmpty()) {
        HighlightsList(dashboard.highlights)
        Spacer(Modifier.height(12.dp))
    }

    dashboard.financial_summary?.let { fin ->
        FinancialSummaryCard(fin)
    }
}

// ---------------------------------------------------------------------------
// Summary card
// ---------------------------------------------------------------------------

@Composable
private fun SummaryCard(text: String) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.primaryContainer,
        ),
        shape = RoundedCornerShape(16.dp),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onPrimaryContainer,
            modifier = Modifier.padding(16.dp),
        )
    }
}

// ---------------------------------------------------------------------------
// Mood & Energy
// ---------------------------------------------------------------------------

@Composable
private fun MoodEnergyRow(mood: Int?, energy: Int?) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        ScoreCard(
            label = "Mood",
            score = mood,
            emoji = moodEmoji(mood),
            modifier = Modifier.weight(1f),
        )
        ScoreCard(
            label = "Energy",
            score = energy,
            emoji = energyEmoji(energy),
            modifier = Modifier.weight(1f),
        )
    }
}

@Composable
private fun ScoreCard(label: String, score: Int?, emoji: String, modifier: Modifier = Modifier) {
    Card(
        modifier = modifier,
        shape = RoundedCornerShape(16.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(emoji, fontSize = 32.sp)
            Spacer(Modifier.height(4.dp))
            Text(
                text = label,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.outline,
            )
            if (score != null) {
                Text(
                    text = "$score/10",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                )
            }
        }
    }
}

private fun moodEmoji(score: Int?): String = when (score) {
    null -> "😶"
    in 1..3 -> "😞"
    in 4..5 -> "😐"
    in 6..7 -> "🙂"
    in 8..9 -> "😊"
    10 -> "🤩"
    else -> "😶"
}

private fun energyEmoji(score: Int?): String = when (score) {
    null -> "🔋"
    in 1..3 -> "🪫"
    in 4..5 -> "🔋"
    in 6..7 -> "⚡"
    in 8..10 -> "🔥"
    else -> "🔋"
}

// ---------------------------------------------------------------------------
// Category breakdown
// ---------------------------------------------------------------------------

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun CategoryBreakdown(categories: Map<String, Int>) {
    val total = categories.values.sum().coerceAtLeast(1)

    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = "Categories",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
            )
            Spacer(Modifier.height(12.dp))

            categories.entries.sortedByDescending { it.value }.forEach { (cat, count) ->
                val fraction = count.toFloat() / total
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 3.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = categoryEmoji(cat),
                        modifier = Modifier.width(24.dp),
                    )
                    Text(
                        text = cat.replace("_", " "),
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.width(90.dp),
                    )
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .height(8.dp)
                            .clip(RoundedCornerShape(4.dp))
                            .background(MaterialTheme.colorScheme.surfaceVariant),
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(fraction)
                                .height(8.dp)
                                .clip(RoundedCornerShape(4.dp))
                                .background(MaterialTheme.colorScheme.primary),
                        )
                    }
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = "$count",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.outline,
                    )
                }
            }
        }
    }
}

private fun categoryEmoji(category: String): String = when (category) {
    "financial" -> "💰"
    "health" -> "💪"
    "idea" -> "💡"
    "learning" -> "📚"
    "relationship" -> "👥"
    "goal" -> "🎯"
    "gratitude" -> "🙏"
    "day_recap" -> "📋"
    "mind_dump" -> "🧠"
    "daily_note" -> "📝"
    else -> "📝"
}

// ---------------------------------------------------------------------------
// Highlights
// ---------------------------------------------------------------------------

@Composable
private fun HighlightsList(highlights: List<Map<String, String>>) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = "Highlights",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
            )
            Spacer(Modifier.height(8.dp))

            highlights.forEach { item ->
                val text = item["text"] ?: return@forEach
                val cat = item["category"]
                Row(
                    modifier = Modifier.padding(vertical = 4.dp),
                    verticalAlignment = Alignment.Top,
                ) {
                    Text(
                        text = if (cat != null) categoryEmoji(cat) else "•",
                        modifier = Modifier.width(24.dp),
                    )
                    Text(
                        text = text,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Financial summary
// ---------------------------------------------------------------------------

@Composable
private fun FinancialSummaryCard(summary: Map<String, Any>) {
    val totalSpent = summary["total_spent"]?.toString() ?: "0"
    val totalEarned = summary["total_earned"]?.toString() ?: "0"

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.tertiaryContainer,
        ),
        shape = RoundedCornerShape(16.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = "💰 Financial",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
            Spacer(Modifier.height(8.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly,
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        text = "Spent",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onTertiaryContainer.copy(alpha = 0.7f),
                    )
                    Text(
                        text = "$$totalSpent",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        text = "Earned",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onTertiaryContainer.copy(alpha = 0.7f),
                    )
                    Text(
                        text = "$$totalEarned",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
    }
}
