package com.seclususs.qos.ui.components.cards

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.seclususs.qos.R

@Composable
fun ErrorStateCard(
    titleResId: Int, descResId: Int, icon: ImageVector
) {
    BaseCard(modifier = Modifier.fillMaxWidth(), onClick = {}, onLongClick = {}) {
        Column(
            modifier = Modifier.padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.error,
                modifier = Modifier.size(48.dp)
            )
            Text(
                text = stringResource(id = titleResId),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center
            )
            Text(
                text = stringResource(id = descResId),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f),
                textAlign = TextAlign.Center
            )
        }
    }
}

@Composable
fun MissingDaemonCard() {
    ErrorStateCard(
        titleResId = R.string.error_daemon_missing_title,
        descResId = R.string.error_daemon_missing_desc,
        icon = Icons.Filled.Warning
    )
}

@Composable
fun MissingConfigCard() {
    ErrorStateCard(
        titleResId = R.string.error_config_missing_title,
        descResId = R.string.error_config_missing_desc,
        icon = Icons.Filled.Error
    )
}