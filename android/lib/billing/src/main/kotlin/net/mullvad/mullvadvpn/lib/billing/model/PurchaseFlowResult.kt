package net.mullvad.mullvadvpn.lib.billing.model

sealed interface PurchaseFlowResult {
    data object Ok: PurchaseFlowResult

    data object UserCancelled: PurchaseFlowResult

    data object BillingUnavailable: PurchaseFlowResult

    data object ServiceUnavailable: PurchaseFlowResult

    data object ItemUnavailable: PurchaseFlowResult

    data class Error(val exception: BillingException): PurchaseFlowResult
}
