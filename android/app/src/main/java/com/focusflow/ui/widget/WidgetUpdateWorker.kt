package com.focusflow.ui.widget

import android.content.Context
import android.util.Log
import androidx.datastore.preferences.core.edit
import androidx.glance.appwidget.GlanceAppWidgetManager
import androidx.glance.appwidget.state.updateAppWidgetState
import androidx.glance.state.PreferencesGlanceStateDefinition
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.focusflow.data.local.FocusFlowDatabase
import java.util.concurrent.TimeUnit

class WidgetUpdateWorker(
    private val context: Context,
    params: WorkerParameters,
) : CoroutineWorker(context, params) {

    override suspend fun doWork(): Result {
        return try {
            val db = FocusFlowDatabase.getInstance(context)
            val widgetState = db.widgetStateDao().getOnce()

            val completedToday = widgetState?.completedToday ?: 0
            val streakDays = widgetState?.streakDays ?: 0
            val message = widgetState?.motivationalMessage ?: "Ready when you are."

            val manager = GlanceAppWidgetManager(context)
            val widget = FocusFlowWidget()
            val glanceIds = manager.getGlanceIds(FocusFlowWidget::class.java)

            glanceIds.forEach { glanceId ->
                updateAppWidgetState(context, PreferencesGlanceStateDefinition, glanceId) { prefs ->
                    prefs.toMutablePreferences().apply {
                        this[FocusFlowWidget.PREF_COMPLETED_TODAY] = completedToday
                        this[FocusFlowWidget.PREF_STREAK_DAYS] = streakDays
                        this[FocusFlowWidget.PREF_MOTIVATIONAL_MESSAGE] = message
                    }
                }
                widget.update(context, glanceId)
            }

            Result.success()
        } catch (e: Exception) {
            Log.e(TAG, "Widget update failed", e)
            Result.retry()
        }
    }

    companion object {
        private const val TAG = "WidgetUpdateWorker"
        private const val PERIODIC_WORK_NAME = "focusflow_widget_update"

        fun schedulePeriodicUpdate(context: Context) {
            val request = PeriodicWorkRequestBuilder<WidgetUpdateWorker>(
                15, TimeUnit.MINUTES,
            )
                .setConstraints(
                    Constraints.Builder()
                        .setRequiredNetworkType(NetworkType.NOT_REQUIRED)
                        .build()
                )
                .build()

            WorkManager.getInstance(context).enqueueUniquePeriodicWork(
                PERIODIC_WORK_NAME,
                ExistingPeriodicWorkPolicy.KEEP,
                request,
            )
        }

        fun triggerImmediateUpdate(context: Context) {
            val request = OneTimeWorkRequestBuilder<WidgetUpdateWorker>().build()
            WorkManager.getInstance(context).enqueue(request)
        }
    }
}
