package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.toPaymentState
import org.joda.time.DateTime

class OutOfTimeViewModel(
    private val accountRepository: AccountRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
    private val deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentUseCase,
    private val pollAccountExpiry: Boolean = true,
) : ViewModel() {

    private val _uiSideEffect = MutableSharedFlow<UiSideEffect>(extraBufferCapacity = 1)
    val uiSideEffect = _uiSideEffect.asSharedFlow()

    val uiState =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    flowOf(state.container)
                } else {
                    emptyFlow()
                }
            }
            .flatMapLatest { serviceConnection ->
                combine(
                    serviceConnection.connectionProxy.tunnelStateFlow(),
                    deviceRepository.deviceState,
                    paymentUseCase.paymentAvailability,
                    paymentUseCase.purchaseResult
                ) { tunnelState, deviceState, paymentAvailability, purchaseResult ->
                    OutOfTimeUiState(
                        tunnelState = tunnelState,
                        deviceName = deviceState.deviceName() ?: "",
                        billingPaymentState =
                            paymentAvailability?.toPaymentState() ?: PaymentState.NoPayment,
                        purchaseResult = purchaseResult
                    )
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), OutOfTimeUiState())

    init {
        viewModelScope.launch {
            accountRepository.accountExpiryState.collectLatest { accountExpiry ->
                accountExpiry.date()?.let { expiry ->
                    val tomorrow = DateTime.now().plusHours(20)

                    if (expiry.isAfter(tomorrow)) {
                        _uiSideEffect.tryEmit(UiSideEffect.OpenConnectScreen)
                    }
                }
            }
        }
        viewModelScope.launch {
            while (pollAccountExpiry) {
                updateAccountExpiry()
                delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
            }
        }
        verifyPurchases(updatePurchaseResult = false)
        fetchPaymentAvailability()
    }

    private fun ConnectionProxy.tunnelStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onStateChange)

    fun onSitePaymentClick() {
        viewModelScope.launch {
            _uiSideEffect.tryEmit(
                UiSideEffect.OpenAccountView(
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    fun onDisconnectClick() {
        viewModelScope.launch { serviceConnectionManager.connectionProxy()?.disconnect() }
    }

    fun startBillingPayment(productId: ProductId, activityProvider: () -> Activity) {
        viewModelScope.launch { paymentUseCase.purchaseProduct(productId, activityProvider) }
    }

    fun verifyPurchases(updatePurchaseResult: Boolean = true) {
        viewModelScope.launch {
            paymentUseCase.verifyPurchases(updatePurchaseResult)
            updateAccountExpiry()
        }
    }

    private fun fetchPaymentAvailability() {
        viewModelScope.launch { paymentUseCase.queryPaymentAvailability() }
    }

    fun onRetryFetchProducts() {
        fetchPaymentAvailability()
    }

    fun onClosePurchaseResultDialog(success: Boolean) {
        // We are closing the dialog without any action, this can happen either if an error occurred
        // during the purchase or the purchase ended successfully.
        // In those cases we want to update the both the payment availability and the account
        // expiry.
        if (success) {
            updateAccountExpiry()
            _uiSideEffect.tryEmit(UiSideEffect.OpenConnectScreen)
        } else {
            fetchPaymentAvailability()
        }
        paymentUseCase.resetPurchaseResult() // So that we do not show the dialog again.
    }

    private fun updateAccountExpiry() {
        accountRepository.fetchAccountExpiry()
    }

    sealed interface UiSideEffect {
        data class OpenAccountView(val token: String) : UiSideEffect

        data object OpenConnectScreen : UiSideEffect
    }
}
