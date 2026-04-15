package com.focusflow.data.local.dao

import androidx.room.Dao
import androidx.room.Query
import androidx.room.Upsert
import com.focusflow.data.local.entity.WidgetStateEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface WidgetStateDao {

    @Query("SELECT * FROM widget_state WHERE id = 1")
    fun get(): Flow<WidgetStateEntity?>

    @Query("SELECT * FROM widget_state WHERE id = 1")
    suspend fun getOnce(): WidgetStateEntity?

    @Upsert
    suspend fun upsert(state: WidgetStateEntity)
}
