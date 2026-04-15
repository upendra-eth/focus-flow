package com.focusflow.domain.models

data class UserSession(
    val userId: String,
    val token: String,
    val deviceId: String
)
