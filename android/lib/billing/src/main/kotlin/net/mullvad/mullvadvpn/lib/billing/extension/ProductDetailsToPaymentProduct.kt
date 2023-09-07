package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.ProductDetails
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct

fun ProductDetails.toPaymentProduct() =
    PaymentProduct(
        productId = this.productId,
        price = this.oneTimePurchaseOfferDetails?.formattedPrice ?: ""
    )

fun List<ProductDetails>.toPaymentProducts() = this.map(ProductDetails::toPaymentProduct)
