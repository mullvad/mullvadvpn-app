package net.mullvad.mullvadvpn.lib.billing.model

sealed interface QueryPurchasesResult {
    data object NoPurchasesFound: QueryPurchasesResult

    data class PurchaseFound(val purchase: BillingPurchase): QueryPurchasesResult

    data class Error(val exception: BillingException): QueryPurchasesResult
}
