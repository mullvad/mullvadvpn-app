package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ButtonGroup
import androidx.compose.material3.ButtonGroupDefaults
import androidx.compose.material3.ButtonGroupScope
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.ToggleButton
import androidx.compose.material3.ToggleButtonColors
import androidx.compose.material3.ToggleButtonShapes
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.onSelected
import net.mullvad.mullvadvpn.lib.theme.color.selected

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Preview
@Composable
private fun PreviewMullvadSegmentedButton() {
    AppTheme {
        Row {
            MullvadSegmentedStartButton(selected = true, text = "Start", onCheckedChange = {})
            MullvadSegmentedMiddleButton(selected = false, text = "Middle", onCheckedChange = {})
            MullvadSegmentedEndButton(selected = false, text = "End", onCheckedChange = {})
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun RowScope.MullvadSegmentedButton(
    selected: Boolean,
    text: String,
    onCheckedChange: (Boolean) -> Unit,
    shapes: ToggleButtonShapes,
) {
    ToggleButton(
        modifier = Modifier.weight(1f),
        onCheckedChange = onCheckedChange,
        checked = selected,
        colors =
            ToggleButtonColors(
                checkedContainerColor = MaterialTheme.colorScheme.selected,
                checkedContentColor = MaterialTheme.colorScheme.onSelected,
                disabledContentColor =
                    MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
                disabledContainerColor =
                    MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
            ),
        border = BorderStroke(0.dp, Color.Unspecified),
        content = {
            Text(
                text = text,
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        shapes = shapes,
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun RowScope.MullvadSegmentedStartButton(
    selected: Boolean,
    text: String,
    onCheckedChange: (Boolean) -> Unit,
) {
    MullvadSegmentedButton(
        selected = selected,
        text = text,
        onCheckedChange = onCheckedChange,
        shapes = ButtonGroupDefaults.connectedLeadingButtonShapes(),
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun RowScope.MullvadSegmentedMiddleButton(
    selected: Boolean,
    text: String,
    onCheckedChange: (Boolean) -> Unit,
) {
    MullvadSegmentedButton(
        selected = selected,
        text = text,
        onCheckedChange = onCheckedChange,
        shapes = ButtonGroupDefaults.connectedMiddleButtonShapes(),
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun RowScope.MullvadSegmentedEndButton(
    selected: Boolean,
    text: String,
    onCheckedChange: (Boolean) -> Unit,
) {
    MullvadSegmentedButton(
        selected = selected,
        text = text,
        onCheckedChange = onCheckedChange,
        shapes = ButtonGroupDefaults.connectedTrailingButtonShapes(),
    )
}
