package net.mullvad.mullvadvpn.lib.payment

import net.mullvad.mullvadvpn.lib.billing.model.BillingProduct

fun BillingProduct.toPaymentProduct() =
    PaymentProduct(productId = this.productId, price = this.price)

fun List<BillingProduct>.toPaymentProducts() = this.map(BillingProduct::toPaymentProduct)
