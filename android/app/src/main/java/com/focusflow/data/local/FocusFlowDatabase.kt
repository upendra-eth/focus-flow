package com.focusflow.data.local

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import com.focusflow.data.local.dao.TaskDao
import com.focusflow.data.local.dao.WidgetStateDao
import com.focusflow.data.local.entity.TaskEntity
import com.focusflow.data.local.entity.WidgetStateEntity

@Database(
    entities = [TaskEntity::class, WidgetStateEntity::class],
    version = 1,
    exportSchema = false
)
abstract class FocusFlowDatabase : RoomDatabase() {

    abstract fun taskDao(): TaskDao
    abstract fun widgetStateDao(): WidgetStateDao

    companion object {
        @Volatile
        private var INSTANCE: FocusFlowDatabase? = null

        fun getInstance(context: Context): FocusFlowDatabase {
            return INSTANCE ?: synchronized(this) {
                val instance = Room.databaseBuilder(
                    context.applicationContext,
                    FocusFlowDatabase::class.java,
                    "focusflow.db"
                ).build()
                INSTANCE = instance
                instance
            }
        }
    }
}
