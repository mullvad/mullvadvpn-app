package net.mullvad.mullvadvpn.lib.payment.model

data class PaymentProduct(val productId: String, val price: String, val status: PaymentStatus)
