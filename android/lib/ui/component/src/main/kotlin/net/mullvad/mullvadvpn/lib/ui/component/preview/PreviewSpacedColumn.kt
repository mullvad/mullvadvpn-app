package net.mullvad.mullvadvpn.lib.ui.component.preview

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
internal fun PreviewSpacedColumn(
    modifier: Modifier = Modifier,
    spacing: Dp = Dimens.listItemDivider,
    verticalAlignment: Alignment.Vertical = Alignment.Bottom,
    horizontalAlignment: Alignment.Horizontal = Alignment.Start,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(spacing, verticalAlignment),
        horizontalAlignment = horizontalAlignment,
        content = content,
    )
}
