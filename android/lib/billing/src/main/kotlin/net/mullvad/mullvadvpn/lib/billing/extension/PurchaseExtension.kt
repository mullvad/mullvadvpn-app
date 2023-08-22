package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.Purchase
import net.mullvad.mullvadvpn.lib.billing.model.BillingPurchase

fun Purchase.toBillingPurchase() =
    BillingPurchase(
        productId = this.products.firstOrNull() ?: "",
        token = this.accountIdentifiers?.obfuscatedAccountId ?: ""
    )
