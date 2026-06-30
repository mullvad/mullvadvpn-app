package net.mullvad.mullvadvpn.feature.home.impl.outoftime

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DisconnectReason
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.payment.util.status
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.PaymentLogic
import net.mullvad.mullvadvpn.lib.usecase.OutOfTimeUseCase

class OutOfTimeViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentLogic,
    private val outOfTimeUseCase: OutOfTimeUseCase,
    private val connectionProxy: ConnectionProxy,
    private val pollAccountExpiry: Boolean = true,
    private val isPlayBuild: Boolean,
) : ViewModel() {

    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect =
        merge(_uiSideEffect.receiveAsFlow(), notOutOfTimeEffect(), revokedDeviceEffect())

    val uiState: StateFlow<Lc<Unit, OutOfTimeUiState>> =
        combine(
                connectionProxy.tunnelState,
                deviceRepository.deviceState.filterNotNull(),
                paymentUseCase.paymentAvailability,
            ) { tunnelState, deviceState, paymentAvailability ->
                Lc.Content(
                    OutOfTimeUiState(
                        tunnelState = tunnelState,
                        deviceName = deviceState.displayName(),
                        showSitePayment = !isPlayBuild,
                        paymentStatus = paymentAvailability?.status(),
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    init {
        viewModelScope.launch {
            while (pollAccountExpiry) {
                paymentUseCase.verifyPurchases()
                updateAccountExpiry()
                delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
            }
        }
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
            connectionProxy.disconnect(DisconnectReason.USER_INITIATED_OUT_OF_TIME).onLeft {
                _uiSideEffect.send(UiSideEffect.GenericError)
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

    private fun revokedDeviceEffect() =
        deviceRepository.deviceState.filterIsInstance<DeviceState.Revoked>().map {
            UiSideEffect.DeviceRevoked
        }

    sealed interface UiSideEffect {
        data class OpenAccountView(val token: WebsiteAuthToken?) : UiSideEffect

        data object OpenConnectScreen : UiSideEffect

        data object DeviceRevoked : UiSideEffect

        data object GenericError : UiSideEffect
    }
}
