package net.mullvad.mullvadvpn.compose.dialog.payment

data class PaymentDialogData(
    val title: Int? = null,
    val message: Int? = null,
    val icon: PaymentDialogIcon,
    val confirmAction: DialogAction? = null,
    val dismissAction: DialogAction? = null,
    val closeOnDismiss: Boolean = true,
    val successfulPayment: Boolean = false,
)

data class DialogAction(
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
