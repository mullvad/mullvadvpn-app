package net.mullvad.mullvadvpn.compose.state

sealed interface OutOfTimeDialogState {
    data object NoDialog: OutOfTimeDialogState

    data object VerificationError: OutOfTimeDialogState

    data object BillingError: OutOfTimeDialogState

    data object PurchaseComplete: OutOfTimeDialogState
}
