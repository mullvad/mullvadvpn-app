package net.mullvad.mullvadvpn.lib.ui.component.button

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.width
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ListItemComponentTokens

/**
 * This composable is meant to be used for icon buttons in list rows. As such the button container
 * fills the max height and has the width given in `containerWidth`. The `modifier` parameter is
 * applied to the inner `IconButton` composable.
 */
@Composable
fun ListItemIconButton(
    modifier: Modifier = Modifier,
    containerWidth: Dp = ListItemComponentTokens.iconContainerWidth,
    onClick: () -> Unit,
    icon: @Composable () -> Unit,
) {
    Box(
        modifier = Modifier.width(containerWidth).fillMaxHeight(),
        contentAlignment = Alignment.Center,
    ) {
        IconButton(
            modifier = modifier,
            onClick = onClick, // { onCellClicked(!isExpanded) },
        ) {
            icon()
        }
    }
}
