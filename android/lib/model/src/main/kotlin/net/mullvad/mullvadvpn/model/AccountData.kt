package net.mullvad.mullvadvpn.model

import org.joda.time.DateTime
import java.util.UUID

data class AccountData(
    val id: UUID,
    val expiryDate: DateTime,
)