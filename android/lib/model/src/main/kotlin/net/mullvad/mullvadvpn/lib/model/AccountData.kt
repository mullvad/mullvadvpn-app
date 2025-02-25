package net.mullvad.mullvadvpn.lib.model

import java.time.ZonedDateTime

data class AccountData(val id: AccountId, val expiryDate: ZonedDateTime)
