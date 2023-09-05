package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager

class VoucherDialogViewModel(
    private val accountRepository: AccountRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
) : ViewModel() {
    private val vmState = MutableStateFlow<String?>(null)

    var uiState =
        vmState
            .map { VoucherDialogUiState(error = it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VoucherDialogUiState(error = null)
            )

    fun onRedeem(voucherCode: String) {
        viewModelScope.launch {
            //            _viewActions.tryEmit(
            //                WelcomeViewModel.ViewAction.OpenAccountView(
            //                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
            //                )
            //            )
        }
    }

    //    sealed interface ViewAction {
    //        data class OpenAccountView(val token: String) : ViewAction
    //
    //        data object OpenConnectScreen : ViewAction
    //    }
}
