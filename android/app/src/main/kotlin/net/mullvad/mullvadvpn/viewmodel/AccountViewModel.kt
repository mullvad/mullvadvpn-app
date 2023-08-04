package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
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

    private val vmState: StateFlow<AccountUiState> =
        combine(deviceRepository.deviceState, accountRepository.accountExpiryState) {
                deviceState,
                accountExpiry ->
                AccountUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date()
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AccountUiState(deviceName = "", accountNumber = "", accountExpiry = null)
            )
    val uiState =
        vmState.stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(),
            AccountUiState(deviceName = "", accountNumber = "", accountExpiry = null)
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

    sealed class ViewAction {
        data class OpenAccountView(val token: String) : ViewAction()
    }
}
