package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Composable
fun SelectableCell(
    title: String,
    isSelected: Boolean,
    titleStyle: TextStyle = MaterialTheme.typography.labelLarge,
    startPadding: Dp = Dimens.indentedCellStartPadding,
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
        startPadding = startPadding
    )
}
