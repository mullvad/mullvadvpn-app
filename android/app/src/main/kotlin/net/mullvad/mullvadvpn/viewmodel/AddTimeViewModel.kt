package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.AddTimeUiState
import net.mullvad.mullvadvpn.compose.state.PurchaseState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.isSuccess
import net.mullvad.mullvadvpn.util.toPaymentState
import net.mullvad.mullvadvpn.viewmodel.AddMoreTimeSideEffect.OpenAccountManagementPageInBrowser

class AddTimeViewModel(
    private val paymentUseCase: PaymentUseCase,
    private val accountRepository: AccountRepository,
    connectionProxy: ConnectionProxy,
    private val isPlayBuild: Boolean,
) : ViewModel() {
    private val _uiSideEffect = Channel<AddMoreTimeSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<Lc<Unit, AddTimeUiState>> =
        combine(
                paymentUseCase.paymentAvailability.filterNotNull(),
                paymentUseCase.purchaseResult,
                connectionProxy.tunnelState,
            ) { paymentAvailability, purchaseResult, tunnelState ->
                Lc.Content(
                    AddTimeUiState(
                        purchaseState = purchaseResult?.toPurchaseState(),
                        billingPaymentState = paymentAvailability.toPaymentState(),
                        tunnelStateBlocked = tunnelState.isBlocked(),
                        showSitePayment = !isPlayBuild,
                    )
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(Unit),
            )

    init {
        verifyPurchases()
        fetchPaymentAvailability()
        handlePurchaseResultTerminatingState()
    }

    fun onManageAccountClick() {
        viewModelScope.launch {
            val wwwAuthToken = accountRepository.getWebsiteAuthToken()
            _uiSideEffect.send(OpenAccountManagementPageInBrowser(wwwAuthToken))
        }
    }

    fun fetchPaymentAvailability() {
        viewModelScope.launch { paymentUseCase.queryPaymentAvailability() }
    }

    fun startBillingPayment(productId: ProductId, activityProvider: () -> Activity) {
        viewModelScope.launch { paymentUseCase.purchaseProduct(productId, activityProvider) }
    }

    fun resetPurchaseResult() {
        viewModelScope.launch { paymentUseCase.resetPurchaseResult() }
    }

    private fun verifyPurchases() {
        viewModelScope.launch {
            if (paymentUseCase.verifyPurchases().isSuccess()) {
                updateAccountExpiry()
            }
        }
    }

    private fun handlePurchaseResultTerminatingState() {
        viewModelScope.launch {
            paymentUseCase.purchaseResult
                .filter { it?.isTerminatingState() == true }
                .collect {
                    // Terminating states are either errors or completed purchases.
                    if (it is PurchaseResult.Completed) {
                        updateAccountExpiry()
                    } else {
                        fetchPaymentAvailability()
                        verifyPurchases() // Attempt to verify again
                    }
                }
        }
    }

    private fun updateAccountExpiry() {
        viewModelScope.launch { accountRepository.refreshAccountData() }
    }

    private fun PurchaseResult.toPurchaseState() =
        when (this) {
            // Idle states
            PurchaseResult.Completed.Cancelled,
            PurchaseResult.BillingFlowStarted,
            is PurchaseResult.Error.BillingError -> {
                // Show nothing
                null
            }
            // Fetching products and obfuscated id loading state
            PurchaseResult.FetchingProducts,
            PurchaseResult.FetchingObfuscationId -> PurchaseState.Connecting
            // Verifying loading states
            PurchaseResult.VerificationStarted -> PurchaseState.VerificationStarted
            // Pending state
            is PurchaseResult.Completed.Pending,
            is PurchaseResult.Error.VerificationError -> PurchaseState.VerifyingPurchase
            // Success state
            is PurchaseResult.Completed.Success -> PurchaseState.Success(productId)
            // Error states
            is PurchaseResult.Error.TransactionIdError ->
                PurchaseState.Error.TransactionIdError(productId = productId)
            is PurchaseResult.Error.FetchProductsError ->
                PurchaseState.Error.OtherError(productId = productId)
            is PurchaseResult.Error.NoProductFound ->
                PurchaseState.Error.OtherError(productId = productId)
        }
}

sealed class AddMoreTimeSideEffect {
    data class OpenAccountManagementPageInBrowser(val token: WebsiteAuthToken?) :
        AddMoreTimeSideEffect()
}
