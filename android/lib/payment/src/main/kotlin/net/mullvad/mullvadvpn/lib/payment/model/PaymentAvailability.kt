package net.mullvad.mullvadvpn.lib.payment.model

sealed interface PaymentAvailability {
    data class ProductsAvailable(val products: List<PaymentProduct>) : PaymentAvailability

    data object ProductsUnavailable : PaymentAvailability

    sealed interface Error : PaymentAvailability {
        data object BillingUnavailable : Error

        data object ServiceUnavailable : Error

        data class Other(val exception: Throwable) : Error
    }
}
