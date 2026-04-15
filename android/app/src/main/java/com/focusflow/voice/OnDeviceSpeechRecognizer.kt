package com.focusflow.voice

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

enum class SpeechState {
    IDLE, LISTENING, PROCESSING, ERROR
}

class OnDeviceSpeechRecognizer(private val context: Context) {

    private var speechRecognizer: SpeechRecognizer? = null

    private val _state = MutableStateFlow(SpeechState.IDLE)
    val state: StateFlow<SpeechState> = _state

    private val _transcript = MutableStateFlow<String?>(null)
    val transcript: StateFlow<String?> = _transcript

    private var onResult: ((String) -> Unit)? = null
    private var onError: ((String) -> Unit)? = null

    val isAvailable: Boolean
        get() = SpeechRecognizer.isRecognitionAvailable(context)

    fun startListening(
        onTranscript: (String) -> Unit,
        onFail: (String) -> Unit = {}
    ) {
        if (!isAvailable) {
            onFail("Speech recognition not available on this device")
            return
        }

        onResult = onTranscript
        onError = onFail

        speechRecognizer?.destroy()
        speechRecognizer = SpeechRecognizer.createSpeechRecognizer(context).apply {
            setRecognitionListener(listener)
        }

        val intent = Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
            putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
            putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
            putExtra(RecognizerIntent.EXTRA_MAX_RESULTS, 1)
            putExtra(RecognizerIntent.EXTRA_SPEECH_INPUT_MINIMUM_LENGTH_MILLIS, 2000L)
            putExtra(RecognizerIntent.EXTRA_SPEECH_INPUT_COMPLETE_SILENCE_LENGTH_MILLIS, 3000L)
        }

        _state.value = SpeechState.LISTENING
        speechRecognizer?.startListening(intent)
    }

    fun stopListening() {
        speechRecognizer?.stopListening()
        _state.value = SpeechState.PROCESSING
    }

    fun destroy() {
        speechRecognizer?.destroy()
        speechRecognizer = null
        _state.value = SpeechState.IDLE
    }

    private val listener = object : RecognitionListener {
        override fun onReadyForSpeech(params: Bundle?) {
            _state.value = SpeechState.LISTENING
        }

        override fun onBeginningOfSpeech() {}

        override fun onRmsChanged(rmsdB: Float) {}

        override fun onBufferReceived(buffer: ByteArray?) {}

        override fun onEndOfSpeech() {
            _state.value = SpeechState.PROCESSING
        }

        override fun onError(error: Int) {
            val message = when (error) {
                SpeechRecognizer.ERROR_NO_MATCH -> "Didn't catch that. Try again?"
                SpeechRecognizer.ERROR_SPEECH_TIMEOUT -> "I didn't hear anything."
                SpeechRecognizer.ERROR_AUDIO -> "Microphone error"
                SpeechRecognizer.ERROR_NETWORK -> "Network issue — working offline"
                SpeechRecognizer.ERROR_NETWORK_TIMEOUT -> "Network timeout"
                else -> "Something went wrong (error $error)"
            }
            _state.value = SpeechState.ERROR
            onError?.invoke(message)
            _state.value = SpeechState.IDLE
        }

        override fun onResults(results: Bundle?) {
            val matches = results?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
            val text = matches?.firstOrNull()
            if (text != null) {
                _transcript.value = text
                onResult?.invoke(text)
            } else {
                onError?.invoke("Didn't catch that. Try again?")
            }
            _state.value = SpeechState.IDLE
        }

        override fun onPartialResults(partialResults: Bundle?) {
            val partial = partialResults
                ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                ?.firstOrNull()
            if (partial != null) {
                _transcript.value = partial
            }
        }

        override fun onEvent(eventType: Int, params: Bundle?) {}
    }
}
