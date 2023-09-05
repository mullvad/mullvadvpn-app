package net.mullvad.mullvadvpn.compose.screen

import android.content.res.Configuration
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.compose.dialog.RedeemVoucherDialog
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialogScreen() {
    AppTheme {
        RedeemVoucherDialogScreen(
            uiState = VoucherDialogUiState(null),
            viewActions = MutableSharedFlow<Unit>().asSharedFlow(),
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Composable
internal fun RedeemVoucherDialogScreen(
    uiState: VoucherDialogUiState,
    viewActions: SharedFlow<Unit>,
    onRedeem: (voucherCode: String) -> Unit,
    onDismiss: () -> Unit
) {
    RedeemVoucherDialog(uiState, viewActions, onRedeem, onDismiss)
}
