package net.mullvad.mullvadvpn.lib.model

import org.joda.time.DateTime

data class RedeemVoucherSuccess(val timeAdded: Long, val newExpiryDate: DateTime)
