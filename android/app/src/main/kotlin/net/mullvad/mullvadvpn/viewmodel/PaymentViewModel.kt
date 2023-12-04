package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterNot
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialogData
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.toPaymentDialogData

class PaymentViewModel(
    private val paymentUseCase: PaymentUseCase,
) : ViewModel() {
    val uiState: StateFlow<PaymentUiState> =
        paymentUseCase.purchaseResult
            .filterNot {
                it is PurchaseResult.Completed.Cancelled || it is PurchaseResult.Error.BillingError
            }
            .map { PaymentUiState(it?.toPaymentDialogData()) }
            .stateIn(viewModelScope, SharingStarted.Lazily, PaymentUiState(PaymentDialogData()))

    val uiSideEffect =
        paymentUseCase.purchaseResult
            .filter {
                it is PurchaseResult.Completed.Cancelled || it is PurchaseResult.Error.BillingError
            }
            .map { PaymentUiSideEffect.PaymentCancelled }

    fun startBillingPayment(productId: ProductId, activityProvider: () -> Activity) {
        viewModelScope.launch { paymentUseCase.purchaseProduct(productId, activityProvider) }
    }
}

data class PaymentUiState(val paymentDialogData: PaymentDialogData?)

sealed interface PaymentUiSideEffect {
    data object PaymentCancelled : PaymentUiSideEffect
}
