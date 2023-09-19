package net.mullvad.mullvadvpn.compose.state

sealed interface AccountDialogState {
    data object NoDialog: AccountDialogState

    data object VerificationError: AccountDialogState

    data object BillingError: AccountDialogState

    data object PurchaseComplete: AccountDialogState
}
