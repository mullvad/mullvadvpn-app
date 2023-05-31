package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Preview
@Composable
private fun PreviewSelectableCell() {
    AppTheme {
        SpacedColumn(Modifier.background(MaterialTheme.colorScheme.background)) {
            SelectableCell(title = "Selected", isSelected = true)
            SelectableCell(title = "Not Selected", isSelected = false)
        }
    }
}

@Composable
fun SelectableCell(
    title: String,
    isSelected: Boolean,
    iconContentDescription: String? = null,
    selectedIcon: @Composable () -> Unit = {
        Icon(
            painter = painterResource(id = R.drawable.icon_tick),
            contentDescription = iconContentDescription,
            tint = MaterialTheme.colorScheme.onPrimary,
            modifier =
                Modifier.padding(end = Dimens.selectableCellTextMargin)
                    .alpha(
                        if (isSelected) {
                            1f
                        } else {
                            0f
                        }
                    )
        )
    },
    titleStyle: TextStyle = MaterialTheme.typography.labelLarge,
    startPadding: Dp = Dimens.cellStartPadding,
    selectedColor: Color = MaterialTheme.colorScheme.surface,
    backgroundColor: Color = MaterialTheme.colorScheme.secondaryContainer,
    onCellClicked: () -> Unit = {},
) {
    BaseCell(
        onCellClicked = onCellClicked,
        title = { BaseCellTitle(title = title, style = titleStyle) },
        background =
            if (isSelected) {
                selectedColor
            } else {
                backgroundColor
            },
        startPadding = startPadding,
        iconView = selectedIcon
    )
}
