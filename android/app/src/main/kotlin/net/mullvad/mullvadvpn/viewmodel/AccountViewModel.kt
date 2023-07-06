package net.mullvad.mullvadvpn.viewmodel

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.extension.openAccountPageInBrowser

class AccountViewModel(
    private var accountRepository: AccountRepository,
    private var deviceRepository: DeviceRepository
) : ViewModel() {
    private val showAccountNumber = MutableStateFlow<Boolean>(false)
    private val vmState: StateFlow<AccountUiState> =
        combine(deviceRepository.deviceState, accountRepository.accountExpiryState) {
                deviceState,
                accountExpiry ->
                AccountUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    showAccountNumber = showAccountNumber.value,
                    accountExpiry = accountExpiry.date()
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AccountUiState(
                    deviceName = "",
                    accountNumber = "",
                    showAccountNumber = false,
                    accountExpiry = null
                )
            )

    val uiState =
        vmState.stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(),
            AccountUiState(
                deviceName = "",
                accountNumber = "",
                showAccountNumber = false,
                accountExpiry = null
            )
        )
    fun onManageAccountClick(context: Context) {
        deviceRepository.deviceState.value.token()?.let { context.openAccountPageInBrowser(it) }
    }
    fun onLogoutClick() {
        accountRepository.logout()
    }
}
