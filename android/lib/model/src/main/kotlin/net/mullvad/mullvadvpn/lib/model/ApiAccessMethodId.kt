package net.mullvad.mullvadvpn.lib.model

import java.util.UUID

@JvmInline
value class ApiAccessMethodId private constructor(val value: UUID) {

    companion object {
        fun fromString(id: String) = ApiAccessMethodId(value = UUID.fromString(id))
    }
}
