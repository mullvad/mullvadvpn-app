package net.mullvad.mullvadvpn.repository.payment

import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.BillingProduct

sealed interface BillingPaymentAvailability {
    data class ProductsAvailable(val products: List<BillingProduct>) : BillingPaymentAvailability

    data object ProductsUnavailable : BillingPaymentAvailability

    sealed interface Error : BillingPaymentAvailability {
        data object BillingUnavailable : BillingPaymentAvailability

        data object ServiceUnavailable : BillingPaymentAvailability

        data class Other(val exception: BillingException) : BillingPaymentAvailability
    }
}
