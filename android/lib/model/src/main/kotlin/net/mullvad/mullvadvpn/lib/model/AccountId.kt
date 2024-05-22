package net.mullvad.mullvadvpn.lib.model

import java.util.UUID

@JvmInline
value class AccountId(val value: UUID) {
    companion object {
        fun fromString(value: String) = AccountId(UUID.fromString(value))
    }
}
