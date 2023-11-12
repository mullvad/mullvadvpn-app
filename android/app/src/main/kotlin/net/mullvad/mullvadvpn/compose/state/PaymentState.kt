package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct

sealed interface PaymentState {
    data object Loading : PaymentState

    data object NoPayment : PaymentState

    data object NoProductsFounds: PaymentState

    data class PaymentAvailable(val products: List<PaymentProduct>) : PaymentState

    sealed interface Error : PaymentState {
        data object Generic : Error

        data object Billing : Error
    }
}
