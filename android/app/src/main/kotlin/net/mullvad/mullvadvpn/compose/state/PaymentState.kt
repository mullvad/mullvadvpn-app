package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct

sealed interface PaymentState {
    data object Loading : PaymentState

    data object NoPayment : PaymentState

    data object GenericError : PaymentState

    data object BillingError : PaymentState

    data class PaymentAvailable(val products: List<PaymentProduct>): PaymentState
}
