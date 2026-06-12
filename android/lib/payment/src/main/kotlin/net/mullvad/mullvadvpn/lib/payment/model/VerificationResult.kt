package net.mullvad.mullvadvpn.lib.payment.model

interface VerificationResult {
    data object NothingToVerify : VerificationResult

    data class Success(val productId: ProductId) : VerificationResult
}
