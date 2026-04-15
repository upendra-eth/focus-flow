package com.focusflow.ui.widget

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.GlanceTheme
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.provideContent
import androidx.glance.background
import androidx.glance.currentState
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.width
import androidx.glance.state.GlanceStateDefinition
import androidx.glance.state.PreferencesGlanceStateDefinition
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import androidx.glance.unit.ColorProvider
import com.focusflow.ui.theme.Blue500
import com.focusflow.ui.theme.Grey500
import com.focusflow.ui.theme.Grey900
import com.focusflow.ui.theme.NearWhite

class FocusFlowWidget : GlanceAppWidget() {

    override val stateDefinition: GlanceStateDefinition<*> = PreferencesGlanceStateDefinition

    override suspend fun provideGlance(context: Context, id: GlanceId) {
        provideContent {
            GlanceTheme {
                WidgetContent()
            }
        }
    }

    @Composable
    private fun WidgetContent() {
        val prefs = currentState<Preferences>()
        val completedToday = prefs[PREF_COMPLETED_TODAY] ?: 0
        val streakDays = prefs[PREF_STREAK_DAYS] ?: 0
        val message = prefs[PREF_MOTIVATIONAL_MESSAGE] ?: "Ready when you are."

        Column(
            modifier = GlanceModifier
                .fillMaxSize()
                .padding(16.dp)
                .background(NearWhite.copy(alpha = 0.95f)),
            verticalAlignment = Alignment.Top,
            horizontalAlignment = Alignment.Start,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "\u2713",
                    style = TextStyle(
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Bold,
                        color = ColorProvider(Blue500),
                    ),
                )
                Spacer(modifier = GlanceModifier.width(8.dp))
                Text(
                    text = if (completedToday > 0) {
                        "$completedToday done today"
                    } else {
                        "Ready when you are"
                    },
                    style = TextStyle(
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Medium,
                        color = ColorProvider(Grey900),
                    ),
                )
            }

            Spacer(modifier = GlanceModifier.height(8.dp))

            if (streakDays > 0) {
                Text(
                    text = "\uD83D\uDD25 $streakDays day streak",
                    style = TextStyle(
                        fontSize = 14.sp,
                        color = ColorProvider(Grey500),
                    ),
                )
                Spacer(modifier = GlanceModifier.height(8.dp))
            }

            Text(
                text = message,
                style = TextStyle(
                    fontSize = 13.sp,
                    color = ColorProvider(Grey500),
                ),
                maxLines = 2,
            )
        }
    }

    companion object {
        val PREF_COMPLETED_TODAY = intPreferencesKey("completed_today")
        val PREF_STREAK_DAYS = intPreferencesKey("streak_days")
        val PREF_MOTIVATIONAL_MESSAGE = stringPreferencesKey("motivational_message")
    }
}
