package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.toPaymentState

class OutOfTimeViewModel(
    private val accountRepository: net.mullvad.mullvadvpn.lib.account.AccountRepository,
    deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentUseCase,
    private val outOfTimeUseCase: OutOfTimeUseCase,
    private val connectionProxy: ConnectionProxy,
    private val pollAccountExpiry: Boolean = true,
    private val isPlayBuild: Boolean
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
                    deviceName = deviceState?.deviceName() ?: "",
                    showSitePayment = !isPlayBuild,
                    billingPaymentState = paymentAvailability?.toPaymentState(),
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), OutOfTimeUiState())

    init {
        viewModelScope.launch {
            while (pollAccountExpiry) {
                updateAccountExpiry()
                delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
            }
        }
        verifyPurchases()
        fetchPaymentAvailability()
    }

    fun onSitePaymentClick() {
        viewModelScope.launch {
            accountRepository.getAccountToken()?.let { accountToken ->
                _uiSideEffect.send(UiSideEffect.OpenAccountView(accountToken))
            }
        }
    }

    fun onDisconnectClick() {
        viewModelScope.launch { connectionProxy.disconnect() }
    }

    private fun verifyPurchases() {
        viewModelScope.launch {
            paymentUseCase.verifyPurchases()
            updateAccountExpiry()
        }
    }

    private fun fetchPaymentAvailability() {
        viewModelScope.launch { paymentUseCase.queryPaymentAvailability() }
    }

    fun onClosePurchaseResultDialog(success: Boolean) {
        // We are closing the dialog without any action, this can happen either if an error occurred
        // during the purchase or the purchase ended successfully.
        // If the payment was successful we want to update the account expiry. If not successful we
        // should check payment availability and verify any purchases to handle potential errors.
        if (success) {
            viewModelScope.launch { updateAccountExpiry() }
        } else {
            fetchPaymentAvailability()
            verifyPurchases() // Attempt to verify again
        }
        viewModelScope.launch {
            paymentUseCase.resetPurchaseResult() // So that we do not show the dialog again.
        }
    }

    private suspend fun updateAccountExpiry() {
        accountRepository.getAccountData()
    }

    private fun notOutOfTimeEffect() =
        outOfTimeUseCase.isOutOfTime
            .filter { it == false }
            .map {
                paymentUseCase.resetPurchaseResult()
                UiSideEffect.OpenConnectScreen
            }

    sealed interface UiSideEffect {
        data class OpenAccountView(val token: AccountToken) : UiSideEffect

        data object OpenConnectScreen : UiSideEffect
    }
}
