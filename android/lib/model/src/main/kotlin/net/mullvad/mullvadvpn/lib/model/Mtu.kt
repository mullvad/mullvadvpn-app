package net.mullvad.mullvadvpn.lib.model

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
            ensure(number in MIN_VALUE..MAX_VALUE) { ParseMtuError.OutOfRange(number) }
            Mtu(number)
        }

        private const val MIN_VALUE = 1280
        private const val MAX_VALUE = 1420
    }
}

sealed interface ParseMtuError {
    data object NotANumber : ParseMtuError

    data class OutOfRange(val number: Int) : ParseMtuError
}
