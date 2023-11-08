package net.mullvad.mullvadvpn.compose.screen

import android.content.res.Configuration
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.compose.dialog.RedeemVoucherDialog
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialogScreen() {
    AppTheme {
        RedeemVoucherDialogScreen(
            uiState = VoucherDialogUiState.INITIAL,
            onVoucherInputChange = {},
            onRedeem = {},
            onDismiss = {},
            voucherValidator = { true }
        )
    }
}

@Composable
internal fun RedeemVoucherDialogScreen(
    uiState: VoucherDialogUiState,
    onVoucherInputChange: (String) -> Unit = {},
    onRedeem: (voucherCode: String) -> Unit,
    onDismiss: (isTimeAdded: Boolean) -> Unit,
    voucherValidator: (voucher: String) -> Boolean
) {
    RedeemVoucherDialog(
        uiState = uiState,
        onVoucherInputChange = onVoucherInputChange,
        onRedeem = onRedeem,
        onDismiss = onDismiss,
        voucherValidator = voucherValidator
    )
}
