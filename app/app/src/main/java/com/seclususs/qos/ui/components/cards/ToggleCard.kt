package com.seclususs.qos.ui.components.cards

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import com.seclususs.qos.ui.components.inputs.SwitchToggle

@Composable
fun ToggleCard(
    title: String,
    icon: ImageVector,
    isToggled: Boolean,
    isProcessing: Boolean,
    onToggle: (Boolean) -> Unit,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    isExpanded: Boolean = false,
    content: @Composable (() -> Unit)? = null
) {
    ExpandableCard(
        title = title,
        icon = icon,
        isExpanded = isExpanded,
        onExpandClick = onClick,
        modifier = modifier,
        subtitle = subtitle,
        trailingContent = {
            SwitchToggle(
                isChecked = isToggled,
                isProcessing = isProcessing,
                onToggle = { if (!isProcessing) onToggle(!isToggled) })
        }) {
        content?.invoke()
    }
}