package net.mullvad.mullvadvpn.model

import java.util.UUID
import org.joda.time.DateTime

data class AccountData(
    val id: UUID,
    val expiryDate: DateTime,
)
