package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RedeemVoucherError

data class VoucherDialogUiState(
    val voucherInput: String = "",
    val voucherState: VoucherDialogState = VoucherDialogState.Default,
) {
    companion object {
        val INITIAL = VoucherDialogUiState()
    }
}

sealed interface VoucherDialogState {

    data object Default : VoucherDialogState

    data object Verifying : VoucherDialogState

    data class Success(val addedTime: Long) : VoucherDialogState

    data class Error(val error: RedeemVoucherError) : VoucherDialogState
}
