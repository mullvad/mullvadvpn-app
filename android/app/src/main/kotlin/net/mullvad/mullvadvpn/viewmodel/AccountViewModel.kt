package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
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
    val uiSideEffect = _uiSideEffect.asSharedFlow()

    val uiState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState,
                accountRepository.accountExpiryState,
                paymentUseCase.paymentAvailability
            ) { deviceState, accountExpiry, paymentAvailability ->
                AccountUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date(),
                    showSitePayment = !IS_PLAY_BUILD,
                    billingPaymentState = paymentAvailability?.toPaymentState()
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState.default())

    init {
        updateAccountExpiry()
        verifyPurchases()
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
        viewModelScope.launch { _uiSideEffect.emit(UiSideEffect.NavigateToLogin) }
    }

    fun onCopyAccountNumber(accountNumber: String) {
        viewModelScope.launch { _uiSideEffect.emit(UiSideEffect.CopyAccountNumber(accountNumber)) }
    }

    fun startBillingPayment(productId: ProductId, activityProvider: () -> Activity) {
        viewModelScope.launch { paymentUseCase.purchaseProduct(productId, activityProvider) }
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
        // In those cases we want to update the both the payment availability and the account
        // expiry.
        if (success) {
            updateAccountExpiry()
        } else {
            fetchPaymentAvailability()
        }
        viewModelScope.launch {
            paymentUseCase.resetPurchaseResult() // So that we do not show the dialog again.
        }
    }

    private fun updateAccountExpiry() {
        accountRepository.fetchAccountExpiry()
    }

    sealed class UiSideEffect {
        data object NavigateToLogin : UiSideEffect()

        data class OpenAccountManagementPageInBrowser(val token: String) : UiSideEffect()

        data class CopyAccountNumber(val accountNumber: String) : UiSideEffect()
    }
}

data class AccountUiState(
    val deviceName: String?,
    val accountNumber: String?,
    val accountExpiry: DateTime?,
    val showSitePayment: Boolean,
    val billingPaymentState: PaymentState? = null,
) {
    companion object {
        fun default() =
            AccountUiState(
                deviceName = DeviceState.Unknown.deviceName(),
                accountNumber = DeviceState.Unknown.token(),
                accountExpiry = AccountExpiry.Missing.date(),
                showSitePayment = false,
                billingPaymentState = PaymentState.Loading,
            )
    }
}
