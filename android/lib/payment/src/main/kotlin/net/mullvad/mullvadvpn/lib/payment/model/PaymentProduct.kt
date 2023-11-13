package net.mullvad.mullvadvpn.lib.payment.model

data class PaymentProduct(
    val productId: ProductId,
    val price: ProductPrice,
    val status: PaymentStatus?
)
