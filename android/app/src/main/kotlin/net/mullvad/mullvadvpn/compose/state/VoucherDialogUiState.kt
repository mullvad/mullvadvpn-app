package net.mullvad.mullvadvpn.compose.state

data class VoucherDialogUiState(
    var message: String? = null,
    var isError: Boolean = false,
    var showLoading: Boolean = false
)

sealed class VoucherDialogViewModelState {
    data object Default : VoucherDialogViewModelState()

    data object Verifying : VoucherDialogViewModelState()

    data class Success(var message: String?) : VoucherDialogViewModelState()

    data class Error(val errorMessage: String) : VoucherDialogViewModelState()

    fun toUiState(): VoucherDialogUiState {
        return when (this) {
            is Default -> VoucherDialogUiState(null, false, false)
            is Verifying -> VoucherDialogUiState("loading", false, true)
            is Success -> VoucherDialogUiState("success", false, false)
            is Error -> VoucherDialogUiState("error", true, false)
        }
    }
}
