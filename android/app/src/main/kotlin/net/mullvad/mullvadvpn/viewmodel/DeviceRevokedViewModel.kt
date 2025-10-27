package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase

class DeviceRevokedViewModel(
    private val accountRepository: AccountRepository,
    private val connectionProxy: ConnectionProxy,
    private val scheduleNotificationAlarmUseCase: ScheduleNotificationAlarmUseCase,
    private val accountExpiryNotificationProvider: AccountExpiryNotificationProvider,
) : ViewModel() {

    val uiState =
        connectionProxy.tunnelState
            .onStart {
                accountExpiryNotificationProvider.cancelNotification()
                scheduleNotificationAlarmUseCase(accountExpiry = null)
            }
            .map {
                if (it.isSecured()) {
                    DeviceRevokedUiState.SECURED
                } else {
                    DeviceRevokedUiState.UNSECURED
                }
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = DeviceRevokedUiState.UNKNOWN,
            )

    private val _uiSideEffect = Channel<DeviceRevokedSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun onGoToLoginClicked() {
        viewModelScope.launch {
            connectionProxy.disconnect()
            accountRepository.logout()
        }

        viewModelScope.launch { _uiSideEffect.send(DeviceRevokedSideEffect.NavigateToLogin) }
    }
}

sealed interface DeviceRevokedSideEffect {
    data object NavigateToLogin : DeviceRevokedSideEffect
}
