package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import kotlinx.parcelize.Parcelize

@JvmInline
@Parcelize
value class Mtu(val value: Int) : Parcelable {
    companion object {
        fun fromString(value: String): Either<ParseMtuError, Mtu> = either {
            val number = value.toIntOrNull() ?: raise(ParseMtuError.NotANumber)
            ensure(number in MIN_MTU..MAX_MTU) { ParseMtuError.OutOfRange(number) }
            Mtu(number)
        }

        const val MIN_MTU = 1280
        const val MAX_MTU = 1420
    }
}

sealed interface ParseMtuError {
    data object NotANumber : ParseMtuError

    data class OutOfRange(val number: Int) : ParseMtuError
}
