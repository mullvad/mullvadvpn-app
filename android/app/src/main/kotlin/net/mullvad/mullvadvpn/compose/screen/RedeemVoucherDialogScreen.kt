package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.runtime.Composable
import net.mullvad.mullvadvpn.compose.dialog.RedeemVoucherDialog
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState

@Composable
internal fun RedeemVoucherDialogScreen(
    uiState: VoucherDialogUiState,
    onRedeem: (voucherCode: String) -> Unit,
    onDismiss: () -> Unit
) {
    RedeemVoucherDialog(uiState, onRedeem, onDismiss)
}
