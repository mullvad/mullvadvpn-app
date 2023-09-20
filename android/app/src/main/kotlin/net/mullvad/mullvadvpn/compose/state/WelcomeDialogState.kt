package net.mullvad.mullvadvpn.compose.state

sealed interface WelcomeDialogState {
    data object NoDialog: WelcomeDialogState

    data object VerificationError: WelcomeDialogState

    data object BillingError: WelcomeDialogState

    data object PurchaseComplete: WelcomeDialogState
}
