package net.mullvad.mullvadvpn.compose.state

data class VoucherDialogUiState(
    val voucherInput: String = "",
    val voucherViewModelState: VoucherDialogState = VoucherDialogState.Default
) {
    companion object {
        val INITIAL = VoucherDialogUiState()
    }
}

sealed interface VoucherDialogState {

    data object Default : VoucherDialogState

    data object Verifying : VoucherDialogState

    data class Success(val addedTime: Long) : VoucherDialogState

    data class Error(val errorMessage: String) : VoucherDialogState
}
