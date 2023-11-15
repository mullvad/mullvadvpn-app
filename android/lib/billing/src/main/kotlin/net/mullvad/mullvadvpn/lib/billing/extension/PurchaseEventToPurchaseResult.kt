package net.mullvad.mullvadvpn.lib.billing.extension

import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult

fun PurchaseEvent.toPurchaseResult() =
    when (this) {
        is PurchaseEvent.Error -> PurchaseResult.Error.BillingError(this.exception)
        is PurchaseEvent.Completed -> PurchaseResult.VerificationStarted
        PurchaseEvent.UserCanceled -> PurchaseResult.Completed.Cancelled
    }
