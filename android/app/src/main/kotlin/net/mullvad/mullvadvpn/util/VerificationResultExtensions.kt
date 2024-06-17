package net.mullvad.mullvadvpn.util

import arrow.core.Either
import net.mullvad.mullvadvpn.lib.payment.model.VerificationError
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

fun Either<VerificationError, VerificationResult>.isSuccess() =
    getOrNull() == VerificationResult.Success
