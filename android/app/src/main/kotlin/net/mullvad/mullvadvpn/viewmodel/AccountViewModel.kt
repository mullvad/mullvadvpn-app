package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.AccountDialogState
import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.BillingPaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.PurchaseResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache

class AccountViewModel(
    private var accountRepository: AccountRepository,
    private var serviceConnectionManager: ServiceConnectionManager,
    private val paymentRepository: PaymentRepository,
    deviceRepository: DeviceRepository
) : ViewModel() {

    init {
        viewModelScope.launch {
            paymentRepository.verifyPurchases()
        }
    }

    private val _dialogState = MutableStateFlow<AccountDialogState>(AccountDialogState.NoDialog)
    private val _purchaseLoading = MutableStateFlow(false)
    private val _viewActions = MutableSharedFlow<ViewAction>(extraBufferCapacity = 1)
    private val _enterTransitionEndAction = MutableSharedFlow<Unit>()
    val viewActions = _viewActions.asSharedFlow()

    private val vmState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState,
                accountRepository.accountExpiryState,
                paymentRepository.productsFlow(),
                _purchaseLoading,
                _dialogState
            ) { deviceState, accountExpiry, paymentAvailability, purchaseLoading, dialogState ->
                AccountUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date(),
                    webPaymentAvailable = paymentAvailability?.webPaymentAvailable ?: false,
                    billingPaymentState =
                        paymentAvailability?.billingPaymentAvailability?.toPaymentState()
                            ?: PaymentState.Loading,
                    purchaseLoading = purchaseLoading,
                    dialogState = dialogState
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState())
    val uiState =
        vmState.stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState())

    val enterTransitionEndAction = _enterTransitionEndAction.asSharedFlow()

    fun onManageAccountClick() {
        viewModelScope.launch {
            _viewActions.tryEmit(
                ViewAction.OpenAccountManagementPageInBrowser(
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
            _purchaseLoading.tryEmit(true)
            val result = paymentRepository.purchaseBillingProduct(productId)
            _purchaseLoading.tryEmit(false)
            when (result) {
                PurchaseResult.PurchaseCancelled -> {
                    // Do nothing
                }
                PurchaseResult.PurchaseCompleted -> {
                    // Show purchase completed dialog
                    _dialogState.tryEmit(AccountDialogState.PurchaseComplete)
                }
                PurchaseResult.PurchaseError -> {
                    // Show purchase error dialog
                    _dialogState.tryEmit(AccountDialogState.PurchaseError)
                }
                PurchaseResult.VerificationError -> {
                    // Show verification error dialog
                    _dialogState.tryEmit(AccountDialogState.VerificationError)
                }
            }
        }
    }

    private fun PaymentRepository.productsFlow() = callbackFlow {
        this.trySend(null)
        this.trySend(this@productsFlow.queryAvailablePaymentTypes())
    }

    private fun BillingPaymentAvailability.toPaymentState(): PaymentState =
        when (this) {
            BillingPaymentAvailability.Error.ServiceUnavailable,
            BillingPaymentAvailability.Error.BillingUnavailable -> PaymentState.BillingError
            is BillingPaymentAvailability.Error.Other -> PaymentState.GenericError
            is BillingPaymentAvailability.ProductsAvailable ->
                PaymentState.PaymentAvailable(products)
            BillingPaymentAvailability.ProductsUnavailable -> PaymentState.NoPayment
        }

    sealed class ViewAction {
        data class OpenAccountManagementPageInBrowser(val token: String) : ViewAction()
    }
}
