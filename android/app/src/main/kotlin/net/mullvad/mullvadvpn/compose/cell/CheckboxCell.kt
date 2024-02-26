package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.compose.component.MullvadCheckbox
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewCheckboxCell() {
    AppTheme { CheckboxCell(providerName = "", checked = false, onCheckedChange = {}) }
}

@Composable
internal fun CheckboxCell(
    modifier: Modifier = Modifier,
    providerName: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    background: Color = MaterialTheme.colorScheme.secondaryContainer,
    startPadding: Dp = Dimens.mediumPadding,
    endPadding: Dp = Dimens.cellEndPadding,
    minHeight: Dp = Dimens.cellHeight
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier =
            modifier
                .clickable { onCheckedChange(!checked) }
                .defaultMinSize(minHeight = minHeight)
                .fillMaxWidth()
                .background(background)
                .padding(start = startPadding, end = endPadding)
    ) {
        MullvadCheckbox(checked = checked, onCheckedChange = onCheckedChange)

        Spacer(modifier = Modifier.size(Dimens.mediumPadding))

        Text(
            text = providerName,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSecondary,
            modifier =
                Modifier.weight(1f)
                    .padding(top = Dimens.mediumPadding, bottom = Dimens.mediumPadding)
        )
    }
}
