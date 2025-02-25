package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.time.ZonedDateTime
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
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

class AccountViewModel(
    private val accountRepository: AccountRepository,
    deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentUseCase,
    private val isPlayBuild: Boolean,
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val isLoggingOut = MutableStateFlow(false)
    private val isLoadingAccountPage = MutableStateFlow(false)

    val uiState: StateFlow<AccountUiState> =
        combine(
                deviceRepository.deviceState.filterIsInstance<DeviceState.LoggedIn>(),
                accountData(),
                paymentUseCase.paymentAvailability,
                isLoggingOut,
                isLoadingAccountPage,
            ) { deviceState, accountData, paymentAvailability, isLoggingOut, isLoadingAccountPage ->
                AccountUiState(
                    deviceName = deviceState.device.displayName(),
                    accountNumber = deviceState.accountNumber,
                    accountExpiry = accountData?.expiryDate,
                    showLogoutLoading = isLoggingOut,
                    showManageAccountLoading = isLoadingAccountPage,
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
        if (isLoadingAccountPage.value) return
        isLoadingAccountPage.value = true

        viewModelScope.launch {
            val wwwAuthToken = accountRepository.getWebsiteAuthToken()
            _uiSideEffect.send(UiSideEffect.OpenAccountManagementPageInBrowser(wwwAuthToken))
            isLoadingAccountPage.value = false
        }
    }

    fun onLogoutClick() {
        if (isLoggingOut.value) return
        isLoggingOut.value = true

        viewModelScope.launch {
            accountRepository
                .logout()
                .also { isLoggingOut.value = false }
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
    val accountExpiry: ZonedDateTime?,
    val showSitePayment: Boolean,
    val billingPaymentState: PaymentState? = null,
    val showLogoutLoading: Boolean = false,
    val showManageAccountLoading: Boolean = false,
) {
    companion object {
        fun default() =
            AccountUiState(
                deviceName = null,
                accountNumber = null,
                accountExpiry = null,
                showLogoutLoading = false,
                showManageAccountLoading = false,
                showSitePayment = false,
                billingPaymentState = PaymentState.Loading,
            )
    }
}
