package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.toPaymentState
import org.joda.time.DateTime

class AccountViewModel(
    private val accountRepository: AccountRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
    private val paymentUseCase: PaymentUseCase,
    deviceRepository: DeviceRepository
) : ViewModel() {

    private val _uiSideEffect = MutableSharedFlow<UiSideEffect>(extraBufferCapacity = 1)
    private val _enterTransitionEndAction = MutableSharedFlow<Unit>()

    val uiSideEffect = _uiSideEffect.asSharedFlow()

    val uiState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState,
                accountRepository.accountExpiryState,
                paymentUseCase.purchaseResult,
                paymentUseCase.paymentAvailability
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

    fun startBillingPayment(productId: ProductId) {
        viewModelScope.launch { paymentUseCase.purchaseProduct(productId) }
    }

    fun verifyPurchases(updatePurchaseResult: Boolean = true) {
        viewModelScope.launch {
            paymentUseCase.verifyPurchases(updatePurchaseResult)
            updateAccountExpiry()
        }
    }

    @OptIn(FlowPreview::class)
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
        } else {
            fetchPaymentAvailability()
        }
        paymentUseCase.resetPurchaseResult() // So that we do not show the dialog again.
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
