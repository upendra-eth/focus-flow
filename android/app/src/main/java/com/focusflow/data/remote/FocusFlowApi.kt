package com.focusflow.data.remote

import okhttp3.MultipartBody
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.Multipart
import retrofit2.http.PATCH
import retrofit2.http.POST
import retrofit2.http.Part
import retrofit2.http.Path
import retrofit2.http.Query

interface FocusFlowApi {

    @POST("api/v1/auth/device")
    suspend fun authenticateDevice(@Body request: DeviceAuthRequest): AuthResponse

    @Multipart
    @POST("api/v1/voice/upload")
    suspend fun uploadVoice(@Part audio: MultipartBody.Part): VoiceResponse

    @GET("api/v1/tasks")
    suspend fun getTasks(): List<TaskDto>

    @POST("api/v1/tasks")
    suspend fun createTask(@Body task: CreateTaskRequest): TaskDto

    @PATCH("api/v1/tasks/{id}")
    suspend fun updateTask(@Path("id") id: String, @Body update: UpdateTaskRequest): TaskDto

    @GET("api/v1/widget/state")
    suspend fun getWidgetState(): WidgetStateDto

    @GET("api/v1/profile/next-question")
    suspend fun getNextQuestion(): QuestionDto?

    @POST("api/v1/profile/answer")
    suspend fun submitAnswer(@Body answer: SubmitAnswerRequest): AnswerDto

    @POST("api/v1/profile/skip")
    suspend fun skipQuestion(@Body skip: SkipQuestionRequest)

    @GET("api/v1/insights/latest")
    suspend fun getLatestInsight(): InsightDto?

    @POST("api/v1/signals")
    suspend fun sendSignals(@Body signals: SendSignalsRequest)

    // --- Agent ---

    @POST("api/v1/agent/chat")
    suspend fun agentChat(@Body request: AgentChatRequest): AgentChatResponse

    @GET("api/v1/agent/entries")
    suspend fun getEntries(
        @Query("date") date: String? = null,
        @Query("category") category: String? = null,
        @Query("limit") limit: Int? = null,
    ): List<LifeEntryDto>

    @GET("api/v1/agent/dashboard")
    suspend fun getDashboard(@Query("date") date: String? = null): DashboardDto?

    @GET("api/v1/agent/conversations")
    suspend fun getConversations(): List<ConversationDto>
}
