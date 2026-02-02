package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Composable
fun SimpleListItem(
    modifier: Modifier = Modifier,
    title: String,
    subtitle: String? = null,
    position: Position = Position.Single,
    onClick: (() -> Unit)? = null,
) {
    MullvadListItem(
        modifier = modifier,
        position = position,
        content = { TitleAndSubtitle(title = title, subtitle = subtitle) },
        onClick = onClick,
    )
}
