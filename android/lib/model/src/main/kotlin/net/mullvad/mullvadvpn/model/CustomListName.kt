package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
@JvmInline
value class CustomListName private constructor(val value: String) : Parcelable {

    override fun toString() = value

    companion object {
        const val MAX_LENGTH = 30

        fun fromString(name: String): CustomListName {
            val trimmedName = name.trim().take(MAX_LENGTH)
            return CustomListName(trimmedName)
        }
    }
}
