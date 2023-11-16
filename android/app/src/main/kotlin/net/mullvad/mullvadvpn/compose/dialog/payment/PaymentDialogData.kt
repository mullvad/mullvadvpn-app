package net.mullvad.mullvadvpn.compose.dialog.payment

import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.payment.model.ProductId

data class PaymentDialogData(
    val title: Int? = null,
    val message: Int? = null,
    val icon: PaymentDialogIcon? = null,
    val confirmAction: PaymentDialogAction? = null,
    val dismissAction: PaymentDialogAction? = null,
    val closeOnDismiss: Boolean = true,
    val successfulPayment: Boolean = false
)

sealed class PaymentDialogAction(val message: Int) {
    data object Close : PaymentDialogAction(R.string.got_it)

    data class RetryPurchase(val productId: ProductId) : PaymentDialogAction(R.string.try_again)
}

enum class PaymentDialogIcon {
    SUCCESS,
    FAIL,
    LOADING
}
