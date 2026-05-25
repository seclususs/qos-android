package com.seclususs.qos.core.utils

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

fun <T> MutableStateFlow<T>.collectPolling(
    scope: CoroutineScope, onStart: () -> Unit, onStop: () -> Unit
) {
    scope.launch {
        this@collectPolling.subscriptionCount.collect { count ->
            if (count > 0) {
                onStart()
            } else {
                onStop()
            }
        }
    }
}