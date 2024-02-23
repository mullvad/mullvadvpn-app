package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth as wrapContentWidth1
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.constant.MTU_MAX_VALUE
import net.mullvad.mullvadvpn.constant.MTU_MIN_VALUE
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewMtuComposeCell() {
    AppTheme { MtuComposeCell(mtuValue = "1300", onEditMtu = {}) }
}

@Composable
fun MtuComposeCell(
    mtuValue: String,
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
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = modifier
    )
}

@Composable
private fun MtuBodyView(mtuValue: String, modifier: Modifier) {
    Row(modifier = modifier.wrapContentWidth1().wrapContentHeight()) {
        Text(
            text = mtuValue.ifEmpty { stringResource(id = R.string.hint_default) },
            color = MaterialTheme.colorScheme.onPrimary
        )
    }
}

@Composable
fun MtuSubtitle(modifier: Modifier = Modifier) {
    BaseSubtitleCell(
        text = stringResource(R.string.wireguard_mtu_footer, MTU_MIN_VALUE, MTU_MAX_VALUE),
        modifier
    )
}
