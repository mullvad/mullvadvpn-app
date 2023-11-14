package net.mullvad.mullvadvpn.compose.dialog.payment

import net.mullvad.mullvadvpn.lib.payment.model.ProductId

data class PaymentDialogData(
    val title: Int? = null,
    val message: Int? = null,
    val icon: PaymentDialogIcon? = null,
    val confirmAction: PaymentDialogAction? = null,
    val dismissAction: PaymentDialogAction? = null,
    val closeOnDismiss: Boolean = true,
    val successfulPayment: Boolean = false,
    val productId: ProductId = ProductId("")
)

data class PaymentDialogAction(
    val message: Int,
    val onClick: PaymentClickAction,
)

enum class PaymentClickAction {
    CLOSE,
    RETRY_PURCHASE
}

enum class PaymentDialogIcon {
    SUCCESS,
    FAIL,
    LOADING
}
