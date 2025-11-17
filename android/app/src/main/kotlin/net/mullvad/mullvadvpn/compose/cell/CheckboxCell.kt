package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.designsystem.Checkbox

@Preview
@Composable
private fun PreviewCheckboxCell() {
    AppTheme { CheckboxCell(title = "1337", checked = false, onCheckedChange = {}) }
}

@Composable
internal fun CheckboxCell(
    modifier: Modifier = Modifier,
    title: String,
    checked: Boolean,
    enabled: Boolean = true,
    onCheckedChange: (Boolean) -> Unit,
    textStyle: TextStyle = MaterialTheme.typography.bodyLarge,
    background: Color = MaterialTheme.colorScheme.surfaceContainerHighest,
    startPadding: Dp = Dimens.smallPadding,
    endPadding: Dp = Dimens.cellEndPadding,
    minHeight: Dp = Dimens.cellHeight,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier =
            modifier
                .clickable(enabled) { onCheckedChange(!checked) }
                .defaultMinSize(minHeight = minHeight)
                .fillMaxWidth()
                .background(background)
                .padding(start = startPadding, end = endPadding),
    ) {
        Checkbox(checked = checked, onCheckedChange = onCheckedChange)

        Text(
            text = title,
            style = textStyle,
            color = MaterialTheme.colorScheme.onSurface,
            modifier =
                Modifier.weight(1f)
                    .padding(top = Dimens.mediumPadding, bottom = Dimens.mediumPadding),
        )
    }
}
