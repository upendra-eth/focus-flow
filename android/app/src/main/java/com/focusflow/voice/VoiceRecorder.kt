package com.focusflow.voice

import android.content.Context
import android.media.MediaRecorder
import android.os.Build
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.io.File

class VoiceRecorder(private val context: Context) {

    private var recorder: MediaRecorder? = null
    private var outputFile: File? = null
    private var timeoutJob: Job? = null
    private var isRecording = false

    fun start(onAutoStop: () -> Unit) {
        if (isRecording) return

        val file = File(context.cacheDir, "voice_${System.currentTimeMillis()}.m4a")
        outputFile = file

        try {
            recorder = createMediaRecorder().apply {
                setAudioSource(MediaRecorder.AudioSource.MIC)
                setOutputFormat(MediaRecorder.OutputFormat.MPEG_4)
                setAudioEncoder(MediaRecorder.AudioEncoder.AAC)
                setAudioSamplingRate(44100)
                setAudioEncodingBitRate(128000)
                setOutputFile(file.absolutePath)
                setMaxDuration(MAX_DURATION_MS)
                prepare()
                start()
            }
            isRecording = true

            timeoutJob = CoroutineScope(Dispatchers.Main).launch {
                delay(MAX_DURATION_MS.toLong())
                if (isRecording) {
                    onAutoStop()
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start recording", e)
            cleanup()
        }
    }

    fun stop(): ByteArray? {
        if (!isRecording) return null

        timeoutJob?.cancel()
        timeoutJob = null

        return try {
            recorder?.apply {
                stop()
                release()
            }
            recorder = null
            isRecording = false

            outputFile?.readBytes().also {
                outputFile?.delete()
                outputFile = null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to stop recording", e)
            cleanup()
            null
        }
    }

    fun cancel() {
        timeoutJob?.cancel()
        timeoutJob = null
        cleanup()
    }

    private fun cleanup() {
        try {
            if (isRecording) {
                recorder?.stop()
            }
            recorder?.release()
        } catch (_: Exception) { }
        recorder = null
        isRecording = false
        outputFile?.delete()
        outputFile = null
    }

    @Suppress("DEPRECATION")
    private fun createMediaRecorder(): MediaRecorder {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            MediaRecorder(context)
        } else {
            MediaRecorder()
        }
    }

    companion object {
        private const val TAG = "VoiceRecorder"
        private const val MAX_DURATION_MS = 60_000
    }
}
