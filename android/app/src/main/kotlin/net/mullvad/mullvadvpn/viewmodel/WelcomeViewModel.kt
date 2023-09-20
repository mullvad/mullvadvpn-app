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
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.PaymentProvider
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.WelcomeDialogState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.lib.common.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.util.UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS
import net.mullvad.mullvadvpn.util.addDebounceForUnknownState
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import org.joda.time.DateTime

@OptIn(FlowPreview::class)
class WelcomeViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
    private val paymentProvider: PaymentProvider,
    private val pollAccountExpiry: Boolean = true
) : ViewModel() {

    private val paymentRepository: PaymentRepository? = paymentProvider.paymentRepository

    private val _dialogState = MutableStateFlow<WelcomeDialogState>(WelcomeDialogState.NoDialog)
    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val _viewActions = MutableSharedFlow<ViewAction>(extraBufferCapacity = 1)
    val viewActions = _viewActions.asSharedFlow()

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
                    serviceConnection.connectionProxy.tunnelUiStateFlow(),
                    deviceRepository.deviceState.debounce {
                        it.addDebounceForUnknownState(UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS)
                    },
                    _paymentAvailability,
                    _dialogState
                ) { tunnelState, deviceState, paymentAvailability, dialogState ->
                    WelcomeUiState(
                        tunnelState = tunnelState,
                        accountNumber = deviceState.token(),
                        deviceName = deviceState.deviceName()?.capitalizeFirstCharOfEachWord(),
                        billingPaymentState =
                            paymentAvailability?.toPaymentState() ?: PaymentState.Loading,
                        dialogState = dialogState
                    )
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), WelcomeUiState())

    init {
        viewModelScope.launch {
            accountRepository.accountExpiryState.collectLatest { accountExpiry ->
                accountExpiry.date()?.let { expiry ->
                    val tomorrow = DateTime.now().plusHours(20)

                    if (expiry.isAfter(tomorrow)) {
                        _viewActions.tryEmit(ViewAction.OpenConnectScreen)
                    }
                }
            }
        }
        viewModelScope.launch {
            while (pollAccountExpiry) {
                accountRepository.fetchAccountExpiry()
                delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
            }
        }
        viewModelScope.launch {
            paymentRepository?.purchaseResult?.collectLatest { result ->
                when (result) {
                    PurchaseResult.PurchaseCancelled -> {
                        // Do nothing
                    }
                    PurchaseResult.PurchaseCompleted -> {
                        // Show completed dialog
                        _dialogState.tryEmit(WelcomeDialogState.PurchaseComplete)
                    }
                    PurchaseResult.PurchaseError -> {
                        // Do nothing, errors that we get from here should be shown by google
                    }
                    PurchaseResult.VerificationError -> {
                        // Show verification error
                        _dialogState.tryEmit(WelcomeDialogState.VerificationError)
                    }
                }
            }
        }
        verifyPurchases()
        fetchPaymentAvailability()
    }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)

    fun onSitePaymentClick() {
        viewModelScope.launch {
            _viewActions.tryEmit(
                ViewAction.OpenAccountView(
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    fun startBillingPayment(productId: String) {
        viewModelScope.launch { paymentRepository?.purchaseBillingProduct(productId) }
    }

    fun closeDialog() {
        viewModelScope.launch { _dialogState.tryEmit(WelcomeDialogState.NoDialog) }
    }

    fun verifyPurchases() {
        viewModelScope.launch { paymentRepository?.verifyPurchases() }
    }

    fun fetchPaymentAvailability() {
        viewModelScope.launch {
            val result =
                paymentRepository?.queryPaymentAvailability()
                    ?: PaymentAvailability.ProductsUnavailable
            _paymentAvailability.tryEmit(result)
            if (
                result is PaymentAvailability.Error.BillingUnavailable ||
                    result is PaymentAvailability.Error.ServiceUnavailable
            ) {
                _dialogState.tryEmit(WelcomeDialogState.BillingError)
            }
        }
    }

    private fun PaymentAvailability.toPaymentState(): PaymentState =
        when (this) {
            PaymentAvailability.Error.ServiceUnavailable,
            PaymentAvailability.Error.BillingUnavailable -> PaymentState.BillingError
            is PaymentAvailability.Error.Other -> PaymentState.GenericError
            is PaymentAvailability.ProductsAvailable -> PaymentState.PaymentAvailable(products)
            PaymentAvailability.ProductsUnavailable -> PaymentState.NoPayment
        }

    sealed interface ViewAction {
        data class OpenAccountView(val token: String) : ViewAction

        data object OpenConnectScreen : ViewAction
    }
}
