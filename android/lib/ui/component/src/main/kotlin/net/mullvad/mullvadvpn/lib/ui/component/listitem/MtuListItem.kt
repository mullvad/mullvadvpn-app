package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewMtuListView() {
    AppTheme { MtuListItem(mtuValue = Mtu(55555), onEditMtu = {}) }
}

@Composable
fun MtuListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    mtuValue: Mtu?,
    onEditMtu: () -> Unit,
    backgroundAlpha: Float = 1f,
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        backgroundAlpha = backgroundAlpha,
        onClick = onEditMtu,
        content = { Text(text = stringResource(R.string.wireguard_mtu)) },
        trailingContent = {
            Text(
                modifier = Modifier.align(Alignment.CenterEnd).padding(Dimens.mediumPadding),
                text = mtuValue?.value?.toString() ?: stringResource(id = R.string.hint_default),
            )
        },
    )
}
