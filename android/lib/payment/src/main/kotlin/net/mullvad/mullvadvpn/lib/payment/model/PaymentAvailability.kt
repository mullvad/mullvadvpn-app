package net.mullvad.mullvadvpn.lib.payment.model

sealed interface PaymentAvailability {
    data object Loading : PaymentAvailability

    data class ProductsAvailable(val products: List<PaymentProduct>) : PaymentAvailability

    data object ProductsUnavailable : PaymentAvailability

    data object NoProductsFounds : PaymentAvailability

    sealed interface Error: PaymentAvailability {
        data object BillingUnavailable : Error

        data object ServiceUnavailable : Error

        data object FeatureNotSupported : Error

        data object DeveloperError : Error

        data object ItemUnavailable : Error

        data class Other(val exception: Throwable) :
            Error
    }
}
