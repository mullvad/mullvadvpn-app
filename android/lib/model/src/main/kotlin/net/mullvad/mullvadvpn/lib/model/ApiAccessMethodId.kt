package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import java.util.UUID
import kotlinx.parcelize.Parcelize

@JvmInline
@Parcelize
value class ApiAccessMethodId private constructor(val value: UUID) : Parcelable {

    companion object {
        fun fromString(id: String) = ApiAccessMethodId(value = UUID.fromString(id))
    }
}
