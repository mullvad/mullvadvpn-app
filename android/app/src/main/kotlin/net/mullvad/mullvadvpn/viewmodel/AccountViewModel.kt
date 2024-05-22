package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.toPaymentState
import org.joda.time.DateTime

class AccountViewModel(
    private val accountRepository: net.mullvad.mullvadvpn.lib.account.AccountRepository,
    private val paymentUseCase: PaymentUseCase,
    deviceRepository: DeviceRepository,
    private val isPlayBuild: Boolean,
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState,
                accountRepository.accountData,
                paymentUseCase.paymentAvailability
            ) { deviceState, accountData, paymentAvailability ->
                AccountUiState(
                    deviceName = deviceState?.deviceName() ?: "",
                    accountNumber = deviceState?.token()?.value ?: "",
                    accountExpiry = accountData?.expiryDate,
                    showSitePayment = !isPlayBuild,
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
            accountRepository.getAccountToken()?.let { accountToken ->
                _uiSideEffect.send(UiSideEffect.OpenAccountManagementPageInBrowser(accountToken))
            }
        }
    }

    fun onLogoutClick() {
        viewModelScope.launch {
            accountRepository.logout()
            _uiSideEffect.send(UiSideEffect.NavigateToLogin)
        }
    }

    fun onCopyAccountNumber(accountNumber: String) {
        viewModelScope.launch { _uiSideEffect.send(UiSideEffect.CopyAccountNumber(accountNumber)) }
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
        // If the payment was successful we want to update the account expiry. If not successful we
        // should check payment availability and verify any purchases to handle potential errors.
        if (success) {
            updateAccountExpiry()
        } else {
            fetchPaymentAvailability()
            verifyPurchases() // Attempt to verify again
        }
        viewModelScope.launch {
            paymentUseCase.resetPurchaseResult() // So that we do not show the dialog again.
        }
    }

    private fun updateAccountExpiry() {
        viewModelScope.launch { accountRepository.getAccountData() }
    }

    sealed class UiSideEffect {
        data object NavigateToLogin : UiSideEffect()

        data class OpenAccountManagementPageInBrowser(val token: AccountToken) : UiSideEffect()

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
                deviceName = null,
                accountNumber = null,
                accountExpiry = null,
                showSitePayment = false,
                billingPaymentState = PaymentState.Loading,
            )
    }
}
