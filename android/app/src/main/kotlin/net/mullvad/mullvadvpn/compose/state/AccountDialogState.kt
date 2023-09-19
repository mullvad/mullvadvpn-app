package net.mullvad.mullvadvpn.compose.state

sealed interface AccountDialogState {
    data object NoDialog: AccountDialogState

    data object VerificationError: AccountDialogState

    data object PurchaseError: AccountDialogState

    data object PurchaseComplete: AccountDialogState
}
