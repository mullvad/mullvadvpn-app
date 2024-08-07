package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle

@Composable
fun HeaderCell(
    text: String,
    modifier: Modifier = Modifier,
    textStyle: TextStyle = MaterialTheme.typography.titleMedium,
    textColor: Color = MaterialTheme.colorScheme.onSurface,
    background: Color = MaterialTheme.colorScheme.surfaceContainerHighest,
) {
    BaseCell(
        headlineContent = {
            BaseCellTitle(
                title = text,
                style = textStyle,
                textColor = textColor,
            )
        },
        modifier = modifier,
        background = background,
        isRowEnabled = false
    )
}
