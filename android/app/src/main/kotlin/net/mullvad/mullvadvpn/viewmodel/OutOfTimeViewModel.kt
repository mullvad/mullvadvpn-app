package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.PaymentProvider
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.constant.PAYMENT_AVAILABILITY_DEBOUNCE
import net.mullvad.mullvadvpn.constant.PAYMENT_AVAILABILITY_DELAY
import net.mullvad.mullvadvpn.constant.PURCHASES_DELAY
import net.mullvad.mullvadvpn.lib.payment.extensions.toPurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.toPaymentState
import org.joda.time.DateTime

class OutOfTimeViewModel(
    private val accountRepository: AccountRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
    private val deviceRepository: DeviceRepository,
    paymentProvider: PaymentProvider,
    private val pollAccountExpiry: Boolean = true,
) : ViewModel() {
    private val paymentRepository = paymentProvider.paymentRepository

    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val _purchaseResult = MutableStateFlow<PurchaseResult?>(null)
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
                    _paymentAvailability,
                    _purchaseResult
                ) { tunnelState, deviceState, paymentAvailability, purchaseResult ->
                    OutOfTimeUiState(
                        tunnelState = tunnelState,
                        deviceName = deviceState.deviceName() ?: "",
                        billingPaymentState = paymentAvailability?.toPaymentState()
                                ?: PaymentState.NoPayment,
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

    fun startBillingPayment(productId: String) {
        viewModelScope.launch {
            paymentRepository?.purchaseBillingProduct(productId)?.collect(_purchaseResult)
        }
    }

    fun verifyPurchases(updatePurchaseResult: Boolean = true) {
        viewModelScope.launch {
            if (updatePurchaseResult) {
                paymentRepository
                    ?.verifyPurchases()
                    ?.map(VerificationResult::toPurchaseResult)
                    ?.collect(_purchaseResult)
            } else {
                paymentRepository?.verifyPurchases()?.collect {
                    if (it == VerificationResult.Success) {
                        // Update the payment availability after a successful verification.
                        // We add a small delay so that the status is correct
                        fetchPaymentAvailability(PURCHASES_DELAY)
                    }
                }
            }
            updateAccountExpiry()
        }
    }

    @OptIn(FlowPreview::class)
    private fun fetchPaymentAvailability(delay: Long = 0L) {
        viewModelScope.launch {
            delay(delay)
            paymentRepository
                ?.queryPaymentAvailability()
                ?.debounce(PAYMENT_AVAILABILITY_DEBOUNCE) // This is added to avoid flickering
                ?.collect(_paymentAvailability)
                ?: run { _paymentAvailability.emit(PaymentAvailability.ProductsUnavailable) }
        }
    }

    fun onTryFetchProductsAgain() {
        fetchPaymentAvailability(PAYMENT_AVAILABILITY_DELAY)
    }

    fun onClosePurchaseResultDialog(success: Boolean) {
        // We are closing the dialog without any action, this can happen either if an error occurred
        // during the purchase or the purchase ended successfully.
        // In those cases we want to update the both the payment availability and the account
        // expiry.
        fetchPaymentAvailability(PAYMENT_AVAILABILITY_DELAY)
        if (success) {
            updateAccountExpiry()
            _uiSideEffect.tryEmit(UiSideEffect.OpenConnectScreen)
        }
        _purchaseResult.tryEmit(null) // So that we do not show the dialog again.
    }

    private fun updateAccountExpiry() {
        accountRepository.fetchAccountExpiry()
    }

    sealed interface UiSideEffect {
        data class OpenAccountView(val token: String) : UiSideEffect

        data object OpenConnectScreen : UiSideEffect
    }
}
