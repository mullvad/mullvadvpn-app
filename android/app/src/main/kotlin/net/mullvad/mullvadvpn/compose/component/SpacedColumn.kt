package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun SpacedColumn(
    modifier: Modifier = Modifier,
    spacing: Dp = Dimens.listItemDivider,
    alignment: Alignment.Vertical = Alignment.Bottom,
    content: @Composable ColumnScope.() -> Unit
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(spacing, alignment),
        content = content
    )
}
