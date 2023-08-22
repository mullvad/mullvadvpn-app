package net.mullvad.mullvadvpn.lib.billing.model

sealed interface QueryProductResult {
    data class Ok(val products: List<BillingProduct>) : QueryProductResult

    data object BillingUnavailable: QueryProductResult

    data object ServiceUnavailable: QueryProductResult

    data object ItemUnavailable: QueryProductResult

    data class Error(val exception: BillingException): QueryProductResult
}
