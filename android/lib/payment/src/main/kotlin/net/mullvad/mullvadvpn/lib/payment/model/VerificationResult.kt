package net.mullvad.mullvadvpn.lib.payment.model

interface VerificationResult {
    data object NothingToVerify : VerificationResult

    data object Success : VerificationResult
}
