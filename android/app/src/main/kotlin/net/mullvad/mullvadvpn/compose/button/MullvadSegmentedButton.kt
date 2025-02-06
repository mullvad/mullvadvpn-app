package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.SingleChoiceSegmentedButtonRowScope
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.onSelected
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Preview
@Composable
private fun PreviewMullvadSegmentedButton() {
    AppTheme {
        SingleChoiceSegmentedButtonRow {
            MullvadSegmentedStartButton(selected = true, text = "Start", onClick = {})
            MullvadSegmentedMiddleButton(selected = false, text = "Middle", onClick = {})
            MullvadSegmentedEndButton(selected = false, text = "End", onClick = {})
        }
    }
}

@Composable
private fun SingleChoiceSegmentedButtonRowScope.MullvadSegmentedButton(
    selected: Boolean,
    text: String,
    onClick: () -> Unit,
    shape: Shape,
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
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        icon = {},
        shape = shape,
    )
}

@Composable
fun SingleChoiceSegmentedButtonRowScope.MullvadSegmentedStartButton(
    selected: Boolean,
    text: String,
    onClick: () -> Unit,
) {
    MullvadSegmentedButton(
        selected = selected,
        text = text,
        onClick = onClick,
        shape = RoundedCornerShape(topStart = 8.dp, bottomStart = 8.dp),
    )
}

@Composable
fun SingleChoiceSegmentedButtonRowScope.MullvadSegmentedMiddleButton(
    selected: Boolean,
    text: String,
    onClick: () -> Unit,
) {
    MullvadSegmentedButton(
        selected = selected,
        text = text,
        onClick = onClick,
        shape = RoundedCornerShape(0.dp), // Square
    )
}

@Composable
fun SingleChoiceSegmentedButtonRowScope.MullvadSegmentedEndButton(
    selected: Boolean,
    text: String,
    onClick: () -> Unit,
) {
    MullvadSegmentedButton(
        selected = selected,
        text = text,
        onClick = onClick,
        shape = RoundedCornerShape(topEnd = 8.dp, bottomEnd = 8.dp),
    )
}
