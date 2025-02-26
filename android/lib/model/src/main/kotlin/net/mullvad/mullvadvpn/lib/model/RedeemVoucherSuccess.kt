package net.mullvad.mullvadvpn.lib.model

import java.time.ZonedDateTime

data class RedeemVoucherSuccess(val timeAdded: Long, val newExpiryDate: ZonedDateTime)
