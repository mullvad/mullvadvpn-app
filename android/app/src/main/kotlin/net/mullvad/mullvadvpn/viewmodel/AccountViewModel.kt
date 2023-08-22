package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache

class AccountViewModel(
    private var accountRepository: AccountRepository,
    private var serviceConnectionManager: ServiceConnectionManager,
    deviceRepository: DeviceRepository
) : ViewModel() {

    private val _viewActions = MutableSharedFlow<ViewAction>(extraBufferCapacity = 1)
    val viewActions = _viewActions.asSharedFlow()

    private val dialogState =
        MutableStateFlow<AccountScreenDialogState>(AccountScreenDialogState.NoDialog)

    private val vmState: StateFlow<AccountViewModelState> =
        combine(deviceRepository.deviceState, accountRepository.accountExpiryState, dialogState) {
                deviceState,
                accountExpiry,
                dialogState ->
                AccountViewModelState(
                    deviceState = deviceState,
                    accountExpiry = accountExpiry,
                    dialogState = dialogState
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AccountViewModelState.default()
            )
    val uiState =
        vmState
            .map(AccountViewModelState::toUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AccountUiState.DefaultUiState()
            )

    fun onManageAccountClick() {
        viewModelScope.launch {
            _viewActions.tryEmit(
                ViewAction.OpenAccountView(
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    fun onLogoutClick() {
        accountRepository.logout()
    }

    fun onDeviceNameInfoClick() {
        dialogState.update { AccountScreenDialogState.DeviceNameInfoDialog }
    }

    fun onDismissInfoClick() {
        hideDialog()
    }

    private fun hideDialog() {
        dialogState.update { AccountScreenDialogState.NoDialog }
    }

    sealed class ViewAction {
        data class OpenAccountView(val token: String) : ViewAction()
    }
}
