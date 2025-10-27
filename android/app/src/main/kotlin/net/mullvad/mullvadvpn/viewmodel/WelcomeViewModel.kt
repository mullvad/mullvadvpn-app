package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.isAfterNowInstant
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.hasPendingPayment
import net.mullvad.mullvadvpn.util.isSuccess

class WelcomeViewModel(
    private val accountRepository: AccountRepository,
    deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentUseCase,
    private val connectionProxy: ConnectionProxy,
    private val pollAccountExpiry: Boolean = true,
    private val isPlayBuild: Boolean,
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow(), hasAddedTimeEffect())

    val uiState =
        combine(
                connectionProxy.tunnelState,
                deviceRepository.deviceState.filterNotNull(),
                paymentUseCase.paymentAvailability,
            ) { tunnelState, accountState, paymentAvailability ->
                Lc.Content(
                    WelcomeUiState(
                        tunnelState = tunnelState,
                        accountNumber = accountState.accountNumber(),
                        deviceName = accountState.displayName(),
                        showSitePayment = !isPlayBuild,
                        verificationPending = paymentAvailability.hasPendingPayment(),
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
                updateAccountExpiry()
                delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
            }
        }
        verifyPurchases()
        viewModelScope.launch { deviceRepository.updateDevice() }
        viewModelScope.launch {
            val accountNumber =
                uiState.map { it.contentOrNull()?.accountNumber }.filterNotNull().first()
            _uiSideEffect.send(UiSideEffect.StoreCredentialsRequest(accountNumber))
        }
    }

    private fun hasAddedTimeEffect() =
        accountRepository.accountData
            .filterNotNull()
            .filter { it.expiryDate.minusHours(MIN_HOURS_PAST_ACCOUNT_EXPIRY).isAfterNowInstant() }
            .onEach {
                paymentUseCase.resetPurchaseResult()
                accountRepository.resetIsNewAccount()
            }
            .map { UiSideEffect.OpenConnectScreen }

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

    sealed interface UiSideEffect {
        data class OpenAccountView(val token: WebsiteAuthToken?) : UiSideEffect

        data object OpenConnectScreen : UiSideEffect

        data class StoreCredentialsRequest(val accountNumber: AccountNumber) : UiSideEffect

        data object GenericError : UiSideEffect
    }

    companion object {
        private const val MIN_HOURS_PAST_ACCOUNT_EXPIRY: Long = 20
    }
}
