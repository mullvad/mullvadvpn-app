package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import net.mullvad.mullvadvpn.relaylist.RelayItem

@Composable
fun CustomListCell(
    customList: RelayItem.CustomList,
    modifier: Modifier = Modifier,
    onCellClicked: (RelayItem.CustomList) -> Unit = {},
    textStyle: TextStyle = MaterialTheme.typography.titleMedium,
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    background: Color = MaterialTheme.colorScheme.primary,
) {
    BaseCell(
        title = {
            BaseCellTitle(
                title = customList.name,
                style = textStyle,
                color = textColor,
            )
        },
        modifier = modifier,
        background = background,
        onCellClicked = { onCellClicked(customList) }
    )
}
