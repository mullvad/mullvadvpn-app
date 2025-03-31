package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.onSelected
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Preview
@Composable
private fun PreviewCustomPortCell() {
    AppTheme {
        SpacedColumn(Modifier.background(MaterialTheme.colorScheme.surface)) {
            CustomPortCell(
                title = "Title",
                isSelected = true,
                port = Port(444),
                onPortCellClicked = {},
                onMainCellClicked = {},
            )
            CustomPortCell(
                title = "Title",
                isSelected = false,
                port = null,
                onPortCellClicked = {},
                onMainCellClicked = {},
            )
        }
    }
}

@Composable
fun CustomPortCell(
    title: String,
    isSelected: Boolean,
    port: Port?,
    modifier: Modifier = Modifier,
    mainTestTag: String = "",
    numberTestTag: String = "",
    isEnabled: Boolean = true,
    onMainCellClicked: () -> Unit,
    onPortCellClicked: () -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Start,
        modifier = modifier.height(Dimens.cellHeight).fillMaxWidth(),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Start,
            modifier =
                Modifier.clickable(enabled = isEnabled) { onMainCellClicked() }
                    .height(Dimens.cellHeight)
                    .weight(1f)
                    .background(
                        if (isSelected) {
                            MaterialTheme.colorScheme.selected
                        } else {
                            MaterialTheme.colorScheme.surfaceContainerLow
                        }
                    )
                    .padding(start = Dimens.cellStartPadding)
                    .testTag(mainTestTag),
        ) {
            SelectableIcon(
                isSelected = isSelected,
                iconContentDescription = null,
                isEnabled = isEnabled,
            )
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.labelLarge,
                textAlign = TextAlign.Start,
                textColor =
                    if (isSelected) {
                            MaterialTheme.colorScheme.onSelected
                        } else {
                            MaterialTheme.colorScheme.onSurface
                        }
                        .copy(alpha = if (isEnabled) AlphaVisible else AlphaDisabled),
            )
        }
        Spacer(modifier = Modifier.width(Dimens.verticalSpacer))
        Box(
            modifier =
                Modifier.clickable(enabled = isEnabled) { onPortCellClicked() }
                    .height(Dimens.cellHeight)
                    .wrapContentWidth()
                    .defaultMinSize(minWidth = Dimens.customPortBoxMinWidth)
                    .background(MaterialTheme.colorScheme.primary)
                    .testTag(numberTestTag)
        ) {
            Text(
                text = port?.value?.toString() ?: stringResource(id = R.string.port),
                color =
                    MaterialTheme.colorScheme.onPrimary.copy(
                        alpha =
                            if (isEnabled) {
                                AlphaVisible
                            } else {
                                AlphaDisabled
                            }
                    ),
                modifier = Modifier.align(Alignment.Center),
            )
        }
    }
}
