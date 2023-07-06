package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository

class AccountViewModel(
    private var accountRepository: AccountRepository,
    private var deviceRepository: DeviceRepository
) : ViewModel() {
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
                AccountUiState(
                    deviceName = "",
                    accountNumber = "",
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
                accountExpiry = null
            )
        )
    fun onLogoutClick() {
        accountRepository.logout()
    }
}
