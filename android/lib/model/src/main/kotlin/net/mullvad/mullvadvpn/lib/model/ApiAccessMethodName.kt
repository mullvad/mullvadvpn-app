package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
@JvmInline
value class ApiAccessMethodName private constructor(val value: String) : Parcelable {
    override fun toString() = value

    companion object {
        const val MAX_LENGTH = 30

        fun fromString(name: String): ApiAccessMethodName {
            val trimmedName = name.trim().take(MAX_LENGTH)
            return ApiAccessMethodName(trimmedName)
        }
    }
}
