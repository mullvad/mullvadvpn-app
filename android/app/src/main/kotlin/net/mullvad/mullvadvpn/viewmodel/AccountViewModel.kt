package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.PaymentProvider
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.constant.PAYMENT_AVAILABILITY_DEBOUNCE
import net.mullvad.mullvadvpn.constant.PAYMENT_AVAILABILITY_DELAY
import net.mullvad.mullvadvpn.constant.PURCHASES_DELAY
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.extensions.toPurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.util.toPaymentState
import org.joda.time.DateTime

class AccountViewModel(
    private var accountRepository: AccountRepository,
    private var serviceConnectionManager: ServiceConnectionManager,
    paymentProvider: PaymentProvider,
    deviceRepository: DeviceRepository
) : ViewModel() {

    private val paymentRepository: PaymentRepository? = paymentProvider.paymentRepository

    private val _uiSideEffect = MutableSharedFlow<UiSideEffect>(extraBufferCapacity = 1)
    private val _enterTransitionEndAction = MutableSharedFlow<Unit>()
    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val _purchaseResult = MutableStateFlow<PurchaseResult?>(null)
    val uiSideEffect = _uiSideEffect.asSharedFlow()

    val uiState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState,
                accountRepository.accountExpiryState,
                _purchaseResult,
                _paymentAvailability
            ) { deviceState, accountExpiry, purchaseResult, paymentAvailability ->
                AccountUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date(),
                    purchaseResult = purchaseResult,
                    billingPaymentState =
                        paymentAvailability?.toPaymentState() ?: PaymentState.Loading
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState.default())

    @Suppress("konsist.ensure public properties use permitted names")
    val enterTransitionEndAction = _enterTransitionEndAction.asSharedFlow()

    init {
        updateAccountExpiry()
        verifyPurchases(updatePurchaseResult = false)
        fetchPaymentAvailability()
    }

    fun onManageAccountClick() {
        viewModelScope.launch {
            _uiSideEffect.tryEmit(
                UiSideEffect.OpenAccountManagementPageInBrowser(
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    fun onLogoutClick() {
        accountRepository.logout()
    }

    fun onTransitionAnimationEnd() {
        viewModelScope.launch { _enterTransitionEndAction.emit(Unit) }
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
        }
        _purchaseResult.tryEmit(null) // So that we do not show the dialog again.
    }

    private fun updateAccountExpiry() {
        accountRepository.fetchAccountExpiry()
    }

    sealed class UiSideEffect {
        data class OpenAccountManagementPageInBrowser(val token: String) : UiSideEffect()
    }
}

data class AccountUiState(
    val deviceName: String?,
    val accountNumber: String?,
    val accountExpiry: DateTime?,
    val billingPaymentState: PaymentState = PaymentState.Loading,
    val purchaseResult: PurchaseResult? = null,
) {
    companion object {
        fun default() =
            AccountUiState(
                deviceName = DeviceState.Unknown.deviceName(),
                accountNumber = DeviceState.Unknown.token(),
                accountExpiry = AccountExpiry.Missing.date(),
                billingPaymentState = PaymentState.Loading,
                purchaseResult = null,
            )
    }
}
