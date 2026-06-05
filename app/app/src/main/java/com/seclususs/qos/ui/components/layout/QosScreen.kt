package com.seclususs.qos.ui.components.layout

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.seclususs.qos.domain.model.SystemStatus
import com.seclususs.qos.ui.components.modifiers.defaultScreenPadding

@Composable
fun QosScreen(
    title: String, systemStatus: SystemStatus = SystemStatus.OK, content: @Composable () -> Unit
) {
    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier.defaultScreenPadding(),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground
            )
            StateAwareContent(systemStatus = systemStatus) {
                content()
            }
        }
    }
}