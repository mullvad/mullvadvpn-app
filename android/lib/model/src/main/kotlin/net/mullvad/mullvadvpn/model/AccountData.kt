package net.mullvad.mullvadvpn.model

import org.joda.time.DateTime

data class AccountData(
    val id: AccountId,
    val expiryDate: DateTime,
)
