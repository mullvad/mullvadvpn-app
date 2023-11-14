package net.mullvad.mullvadvpn.lib.payment.extensions

import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

fun VerificationResult.toPurchaseResult(): PurchaseResult? =
    when (this) {
        is VerificationResult.Error.BillingError ->
            PurchaseResult.Error.BillingError(this.exception)
        is VerificationResult.Error.VerificationError ->
            PurchaseResult.Error.VerificationError(this.exception)
        VerificationResult.FetchingUnfinishedPurchases -> PurchaseResult.VerificationStarted
        VerificationResult.NothingToVerify -> null
        VerificationResult.Success -> PurchaseResult.Completed.Success
        VerificationResult.VerificationStarted -> PurchaseResult.VerificationStarted
    }
