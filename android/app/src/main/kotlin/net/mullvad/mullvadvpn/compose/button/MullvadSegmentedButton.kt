package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRowScope
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.color.onSelected
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Composable
fun SingleChoiceSegmentedButtonRowScope.MullvadSegmentedButton(
    selected: Boolean,
    text: String,
    onClick: () -> Unit,
    position: SegmentedButtonPosition,
) {
    SegmentedButton(
        onClick = onClick,
        selected = selected,
        colors =
            SegmentedButtonDefaults.colors()
                .copy(
                    activeContainerColor = MaterialTheme.colorScheme.selected,
                    activeContentColor = MaterialTheme.colorScheme.onSelected,
                    inactiveContainerColor = MaterialTheme.colorScheme.primary,
                    inactiveContentColor = MaterialTheme.colorScheme.onPrimary,
                ),
        border = BorderStroke(0.dp, Color.Unspecified),
        label = {
            Text(
                text = text,
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        icon = {},
        shape =
            when (position) {
                SegmentedButtonPosition.First ->
                    RoundedCornerShape(topStart = 8.dp, bottomStart = 8.dp)
                SegmentedButtonPosition.Last -> RoundedCornerShape(topEnd = 8.dp, bottomEnd = 8.dp)
                SegmentedButtonPosition.Middle -> RoundedCornerShape(0.dp) // Square
            },
    )
}

sealed interface SegmentedButtonPosition {
    data object First : SegmentedButtonPosition

    data object Middle : SegmentedButtonPosition

    data object Last : SegmentedButtonPosition
}
