package com.focusflow

import android.app.Application
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import com.focusflow.data.local.FocusFlowDatabase
import com.focusflow.data.sync.SyncWorker
import java.util.concurrent.TimeUnit

class FocusFlowApp : Application() {

    lateinit var database: FocusFlowDatabase
        private set

    override fun onCreate() {
        super.onCreate()
        instance = this

        database = FocusFlowDatabase.getInstance(this)

        setupPeriodicSync()
    }

    private fun setupPeriodicSync() {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(NetworkType.CONNECTED)
            .build()

        val syncRequest = PeriodicWorkRequestBuilder<SyncWorker>(
            15, TimeUnit.MINUTES
        )
            .setConstraints(constraints)
            .build()

        WorkManager.getInstance(this).enqueueUniquePeriodicWork(
            SyncWorker.WORK_NAME,
            ExistingPeriodicWorkPolicy.KEEP,
            syncRequest
        )
    }

    companion object {
        lateinit var instance: FocusFlowApp
            private set
    }
}
