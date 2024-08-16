package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.constant.MTU_MAX_VALUE
import net.mullvad.mullvadvpn.constant.MTU_MIN_VALUE
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewMtuComposeCell() {
    AppTheme { MtuComposeCell(mtuValue = Mtu(1300), onEditMtu = {}) }
}

@Composable
fun MtuComposeCell(
    mtuValue: Mtu?,
    onEditMtu: () -> Unit,
) {
    val titleModifier = Modifier

    BaseCell(
        headlineContent = { MtuTitle(modifier = titleModifier.weight(1f, true)) },
        bodyView = { MtuBodyView(mtuValue = mtuValue, modifier = titleModifier) },
        onCellClicked = { onEditMtu.invoke() }
    )
}

@Composable
private fun MtuTitle(modifier: Modifier) {
    Text(
        text = stringResource(R.string.wireguard_mtu),
        style = MaterialTheme.typography.titleMedium,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = modifier
    )
}

@Composable
private fun MtuBodyView(mtuValue: Mtu?, modifier: Modifier) {
    Row(modifier = modifier) {
        Text(
            text = mtuValue?.value?.toString() ?: stringResource(id = R.string.hint_default),
            color = MaterialTheme.colorScheme.onSurface
        )
    }
}

@Composable
fun MtuSubtitle(modifier: Modifier = Modifier) {
    BaseSubtitleCell(
        text = stringResource(R.string.wireguard_mtu_footer, MTU_MIN_VALUE, MTU_MAX_VALUE),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = modifier
    )
}
