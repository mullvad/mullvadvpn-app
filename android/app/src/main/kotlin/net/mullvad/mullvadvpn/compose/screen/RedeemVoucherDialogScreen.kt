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
            uiState = VoucherDialogUiState(null),
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Composable
internal fun RedeemVoucherDialogScreen(
    uiState: VoucherDialogUiState,
    onRedeem: (voucherCode: String) -> Unit,
    onDismiss: () -> Unit
) {
    RedeemVoucherDialog(uiState, onRedeem, onDismiss)
}
