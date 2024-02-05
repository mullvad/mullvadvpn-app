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
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    background: Color = MaterialTheme.colorScheme.primary,
) {
    BaseCell(
        title = {
            BaseCellTitle(
                title = text,
                style = textStyle,
                color = textColor,
            )
        },
        modifier = modifier,
        background = background,
        isRowEnabled = false
    )
}
