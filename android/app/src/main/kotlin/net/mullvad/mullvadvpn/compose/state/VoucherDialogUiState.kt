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
            is Default -> VoucherDialogUiState(message = null, isError = false, showLoading = false)
            is Verifying ->
                VoucherDialogUiState(message = null, isError = false, showLoading = true)
            is Success ->
                VoucherDialogUiState(message = message, isError = false, showLoading = false)
            is Error ->
                VoucherDialogUiState(message = errorMessage, isError = true, showLoading = false)
        }
    }
}
