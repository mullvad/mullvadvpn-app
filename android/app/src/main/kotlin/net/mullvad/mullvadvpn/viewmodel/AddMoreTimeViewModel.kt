package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.AddMoreTimeUiState
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.util.isSuccess
import net.mullvad.mullvadvpn.util.toPaymentState
import net.mullvad.mullvadvpn.viewmodel.AddMoreTimeSideEffect.OpenAccountManagementPageInBrowser

class AddMoreTimeViewModel(
    private val paymentUseCase: PaymentUseCase,
    private val accountRepository: AccountRepository,
    private val isPlayBuild: Boolean,
) : ViewModel() {
    private val _uiSideEffect = Channel<AddMoreTimeSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val isLoadingAccountPageButton = MutableStateFlow(false)

    val uiState: StateFlow<Lce<Unit, AddMoreTimeUiState, Unit>> =
        combine(paymentUseCase.paymentAvailability, isLoadingAccountPageButton) {
                paymentAvailability,
                isLoadingAccountPage ->
                Lce.Content(
                    AddMoreTimeUiState(
                        billingPaymentState = paymentAvailability?.toPaymentState(),
                        showSitePayment = !isPlayBuild,
                        showManageAccountLoading = isLoadingAccountPage,
                    )
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                initialValue = Lce.Loading(Unit),
            )

    init {
        verifyPurchases()
        fetchPaymentAvailability()
    }

    fun onManageAccountClick() {
        if (isLoadingAccountPageButton.value) return
        isLoadingAccountPageButton.value = true

        viewModelScope.launch {
            val wwwAuthToken = accountRepository.getWebsiteAuthToken()
            _uiSideEffect.send(OpenAccountManagementPageInBrowser(wwwAuthToken))
            isLoadingAccountPageButton.value = false
        }
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
}

sealed class AddMoreTimeSideEffect {
    data class OpenAccountManagementPageInBrowser(val token: WebsiteAuthToken?) :
        AddMoreTimeSideEffect()

    data object GenericError : AddMoreTimeSideEffect()
}
