package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.hasPendingPayment
import net.mullvad.mullvadvpn.util.isSuccess

class OutOfTimeViewModel(
    private val accountRepository: AccountRepository,
    deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentUseCase,
    private val outOfTimeUseCase: OutOfTimeUseCase,
    private val connectionProxy: ConnectionProxy,
    private val pollAccountExpiry: Boolean = true,
    private val isPlayBuild: Boolean,
) : ViewModel() {

    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow(), notOutOfTimeEffect())

    val uiState =
        combine(
                connectionProxy.tunnelState,
                deviceRepository.deviceState,
                paymentUseCase.paymentAvailability,
            ) { tunnelState, deviceState, paymentAvailability ->
                OutOfTimeUiState(
                    tunnelState = tunnelState,
                    deviceName = deviceState?.displayName() ?: "",
                    showSitePayment = !isPlayBuild,
                    verificationPending = paymentAvailability.hasPendingPayment(),
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                OutOfTimeUiState(),
            )

    init {
        viewModelScope.launch {
            while (pollAccountExpiry) {
                updateAccountExpiry()
                delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
            }
        }
        verifyPurchases()
        viewModelScope.launch { deviceRepository.updateDevice() }
    }

    fun onSitePaymentClick() {
        viewModelScope.launch {
            val wwwAuthToken = accountRepository.getWebsiteAuthToken()
            _uiSideEffect.send(UiSideEffect.OpenAccountView(wwwAuthToken))
        }
    }

    fun onDisconnectClick() {
        viewModelScope.launch {
            connectionProxy.disconnect().onLeft { _uiSideEffect.send(UiSideEffect.GenericError) }
        }
    }

    private fun verifyPurchases() {
        viewModelScope.launch {
            if (paymentUseCase.verifyPurchases().isSuccess()) {
                updateAccountExpiry()
            }
        }
    }

    private suspend fun updateAccountExpiry() {
        accountRepository.refreshAccountData()
    }

    private fun notOutOfTimeEffect() =
        outOfTimeUseCase.isOutOfTime
            .filter { it == false }
            .map {
                paymentUseCase.resetPurchaseResult()
                UiSideEffect.OpenConnectScreen
            }

    sealed interface UiSideEffect {
        data class OpenAccountView(val token: WebsiteAuthToken?) : UiSideEffect

        data object OpenConnectScreen : UiSideEffect

        data object GenericError : UiSideEffect
    }
}
