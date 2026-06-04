package com.seclususs.qos.core.utils

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlin.time.Duration.Companion.milliseconds

fun <T> MutableStateFlow<T>.collectPolling(
    scope: CoroutineScope, intervalMs: Long = 1000L, action: suspend () -> Unit
) {
    var pollingJob: Job? = null
    scope.launch {
        subscriptionCount.collect { count ->
            if (count > 0) {
                if (pollingJob?.isActive != true) {
                    pollingJob = scope.launch {
                        action()
                        while (isActive) {
                            delay(intervalMs.milliseconds)
                            action()
                        }
                    }
                }
            } else {
                pollingJob?.cancel()
                pollingJob = null
            }
        }
    }
}