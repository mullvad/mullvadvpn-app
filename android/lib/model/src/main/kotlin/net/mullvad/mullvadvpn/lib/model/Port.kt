package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import kotlinx.parcelize.Parcelize

@JvmInline
@Parcelize
value class Port(val value: Int) : Parcelable {

    override fun toString(): String = value.toString()

    operator fun rangeTo(other: Port): PortRange = PortRange(value..other.value)

    companion object {
        fun fromString(value: String): Either<ParsePortError, Port> = either {
            val number = value.toIntOrNull() ?: raise(ParsePortError.NotANumber(value))
            ensure(number in MIN_VALUE..MAX_VALUE) { ParsePortError.OutOfRange(number) }
            Port(number)
        }

        const val MIN_VALUE = 0
        const val MAX_VALUE = 65535
    }
}

sealed interface ParsePortError {
    data class NotANumber(val input: String) : ParsePortError

    data class OutOfRange(val value: Int) : ParsePortError
}
