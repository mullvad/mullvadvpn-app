package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Column
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.theme.AppTheme
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
        onClick = onEditMtu,
        backgroundAlpha = backgroundAlpha,
        content = {
            Column {
                Text(text = stringResource(R.string.mtu))
                Text(
                    stringResource(
                        id = R.string.mtu_x,
                        mtuValue?.value?.toString() ?: stringResource(id = R.string.hint_default),
                    ),
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        },
        trailingContent = { Icon(imageVector = Icons.Default.Edit, contentDescription = null) },
    )
}
