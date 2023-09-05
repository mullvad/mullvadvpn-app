package net.mullvad.mullvadvpn.compose.state

sealed class VoucherDialogUiState {
    data object Default : VoucherDialogUiState()

    data object Verifying : VoucherDialogUiState()

    data class Success(var message: String) : VoucherDialogUiState()

    data class Error(val errorMessage: String) : VoucherDialogUiState()
}
