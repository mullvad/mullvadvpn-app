package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import org.joda.time.DateTime

class AccountViewModel(
    private var accountRepository: AccountRepository,
    private var serviceConnectionManager: ServiceConnectionManager,
    deviceRepository: DeviceRepository
) : ViewModel() {

    private val _viewActions = MutableSharedFlow<ViewAction>(extraBufferCapacity = 1)
    private val _enterTransitionEndAction = MutableSharedFlow<Unit>()
    val viewActions = _viewActions.asSharedFlow()

    val uiState =
        combine(deviceRepository.deviceState, accountRepository.accountExpiryState) {
                deviceState,
                accountExpiry ->
                AccountUiState(
                    deviceName = deviceState.deviceName(),
                    accountNumber = deviceState.token(),
                    accountExpiry = accountExpiry.date()
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState.default())

    val enterTransitionEndAction = _enterTransitionEndAction.asSharedFlow()

    fun onManageAccountClick() {
        viewModelScope.launch {
            _viewActions.tryEmit(
                ViewAction.OpenAccountManagementPageInBrowser(
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    fun onLogoutClick() {
        accountRepository.logout()
    }

    fun onTransitionAnimationEnd() {
        viewModelScope.launch { _enterTransitionEndAction.emit(Unit) }
    }

    sealed class ViewAction {
        data class OpenAccountManagementPageInBrowser(val token: String) : ViewAction()
    }
}

data class AccountUiState(
    val deviceName: String?,
    val accountNumber: String?,
    val accountExpiry: DateTime?
) {
    companion object {
        fun default() =
            AccountUiState(
                deviceName = DeviceState.Unknown.deviceName(),
                accountNumber = DeviceState.Unknown.token(),
                accountExpiry = AccountExpiry.Missing.date()
            )
    }
}
