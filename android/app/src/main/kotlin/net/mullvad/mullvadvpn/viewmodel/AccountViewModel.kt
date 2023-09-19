package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.PaymentProvider
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache

class AccountViewModel(
    private var accountRepository: AccountRepository,
    private var serviceConnectionManager: ServiceConnectionManager,
    paymentProvider: PaymentProvider,
    deviceRepository: DeviceRepository
) : ViewModel() {

    private val paymentRepository: PaymentRepository? = paymentProvider.paymentRepository

    private val _viewActions = MutableSharedFlow<ViewAction>(extraBufferCapacity = 1)
    private val _enterTransitionEndAction = MutableSharedFlow<Unit>()
    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    val viewActions = _viewActions.asSharedFlow()

    private val _dialogState =
        MutableStateFlow<AccountScreenDialogState>(AccountScreenDialogState.NoDialog)

    private val vmState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState,
                accountRepository.accountExpiryState,
                _dialogState,
                _paymentAvailability
            ) { deviceState, accountExpiry, dialogState, paymentAvailability ->
                AccountUiState(
                    deviceName = deviceState.deviceName(),
                    accountNumber = deviceState.token(),
                    accountExpiry = accountExpiry.date(),
                    dialogState = dialogState,
                    billingPaymentState =
                        paymentAvailability?.toPaymentState() ?: PaymentState.Loading
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState.default())
    val uiState =
        vmState.stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState.default())

    val enterTransitionEndAction = _enterTransitionEndAction.asSharedFlow()

    init {
        verifyPurchases()
        fetchPaymentAvailability()
        viewModelScope.launch {
            paymentRepository?.purchaseResult?.collectLatest { result ->
                when (result) {
                    PurchaseResult.PurchaseCancelled -> {
                        // Do nothing
                    }
                    PurchaseResult.PurchaseCompleted -> {
                        // Show completed dialog
                        _dialogState.tryEmit(AccountScreenDialogState.PurchaseComplete)
                    }
                    PurchaseResult.PurchaseError -> {
                        // Do nothing, errors that we get from here should be shown by google
                    }
                    PurchaseResult.VerificationError -> {
                        // Show verification error
                        _dialogState.tryEmit(AccountScreenDialogState.VerificationError)
                    }
                }
            }
        }
    }

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

    fun onDeviceNameInfoClick() {
        _dialogState.update { AccountScreenDialogState.DeviceNameInfoDialog }
    }

    fun closeDialog() {
        _dialogState.update { AccountScreenDialogState.NoDialog }
    }

    fun onTransitionAnimationEnd() {
        viewModelScope.launch { _enterTransitionEndAction.emit(Unit) }
    }

    fun startBillingPayment(productId: String) {
        viewModelScope.launch { paymentRepository?.purchaseBillingProduct(productId) }
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
                _dialogState.tryEmit(AccountScreenDialogState.BillingError)
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

    sealed class ViewAction {
        data class OpenAccountManagementPageInBrowser(val token: String) : ViewAction()
    }
}
