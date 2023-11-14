package net.mullvad.mullvadvpn.compose.dialog.payment

data class PaymentDialogData(
    val title: Int? = null,
    val message: Int? = null,
    val icon: PaymentDialogIcon? = null,
    val confirmAction: PaymentDialogAction? = null,
    val dismissAction: PaymentDialogAction? = null,
    val closeOnDismiss: Boolean = true,
    val successfulPayment: Boolean = false,
)

data class PaymentDialogAction(
    val message: Int,
    val onClick: PaymentClickAction,
)

enum class PaymentClickAction {
    CLOSE,
    CLOSE_PAYMENT_ERROR,
    RETRY_VERIFICATION,
    RETRY_FETCH_PRODUCTS
}

enum class PaymentDialogIcon {
    SUCCESS,
    FAIL,
    LOADING
}
