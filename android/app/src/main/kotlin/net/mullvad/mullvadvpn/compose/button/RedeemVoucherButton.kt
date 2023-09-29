package net.mullvad.mullvadvpn.compose.button

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewRedeemVoucherButton() {
    AppTheme {
        SpacedColumn {
            RedeemVoucherButton(onClick = {}, isEnabled = true)
            RedeemVoucherButton(onClick = {}, isEnabled = false)
        }
    }
}

@Composable
fun RedeemVoucherButton(
    modifier: Modifier = Modifier,
    background: Color = MaterialTheme.colorScheme.background,
    onClick: () -> Unit,
    isEnabled: Boolean
) {
    VariantButton(
        background = background,
        text = stringResource(id = R.string.redeem_voucher),
        onClick = onClick,
        modifier = modifier,
        isEnabled = isEnabled
    )
}
