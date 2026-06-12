package net.mullvad.mullvadvpn.feature.addtime.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

@Parcelize data class VerificationPendingNavKey(val paymentStatus: PaymentStatus) : NavKey2
