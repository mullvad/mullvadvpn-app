package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.isSuccess
import net.mullvad.mullvadvpn.util.toPaymentState
import org.joda.time.DateTime

class AccountViewModel(
    private val accountRepository: AccountRepository,
    deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentUseCase,
    private val isPlayBuild: Boolean,
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState.filterIsInstance<DeviceState.LoggedIn>(),
                accountData(),
                paymentUseCase.paymentAvailability,
            ) { deviceState, accountData, paymentAvailability ->
                AccountUiState(
                    deviceName = deviceState.device.displayName(),
                    accountNumber = deviceState.accountNumber,
                    accountExpiry = accountData?.expiryDate,
                    showSitePayment = !isPlayBuild,
                    billingPaymentState = paymentAvailability?.toPaymentState(),
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), AccountUiState.default())

    init {
        updateAccountExpiry()
        verifyPurchases()
        fetchPaymentAvailability()
    }

    private fun accountData(): Flow<AccountData?> =
        // Ignore nulls expect first, to avoid loading when logging out.
        accountRepository.accountData
            .filterNotNull()
            .onStart<AccountData?> { emit(accountRepository.accountData.value) }
            .distinctUntilChanged()

    fun onManageAccountClick() {
        viewModelScope.launch {
            val wwwAuthToken = accountRepository.getWebsiteAuthToken()
            _uiSideEffect.send(UiSideEffect.OpenAccountManagementPageInBrowser(wwwAuthToken))
        }
    }

    fun onLogoutClick() {
        viewModelScope.launch {
            accountRepository
                .logout()
                .fold(
                    { _uiSideEffect.send(UiSideEffect.GenericError) },
                    { _uiSideEffect.send(UiSideEffect.NavigateToLogin) },
                )
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
            if (paymentUseCase.verifyPurchases().isSuccess()) {
                updateAccountExpiry()
            }
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

        data class OpenAccountManagementPageInBrowser(val token: WebsiteAuthToken?) :
            UiSideEffect()

        data class CopyAccountNumber(val accountNumber: String) : UiSideEffect()

        data object GenericError : UiSideEffect()
    }
}

data class AccountUiState(
    val deviceName: String?,
    val accountNumber: AccountNumber?,
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
