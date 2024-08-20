package net.mullvad.mullvadvpn.lib.model

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure

@JvmInline
value class VoucherCode private constructor(val value: String) {

    companion object {
        // Parsing reference:
        // <services-repository>/services/docs/adr/0018-distinguish-voucher-codes-from-account-numbers.md
        fun fromString(value: String): Either<ParseVoucherCodeError, VoucherCode> = either {
            val trimmedValue = value.trim()
            ensure(trimmedValue.length >= MIN_VOUCHER_LENGTH) {
                ParseVoucherCodeError.TooShort(trimmedValue)
            }
            ensure(!value.all { it.isDigit() }) { ParseVoucherCodeError.AllDigit(trimmedValue) }
            VoucherCode(trimmedValue)
        }

        const val MIN_VOUCHER_LENGTH = 16
    }
}

sealed interface ParseVoucherCodeError {

    data class AllDigit(val value: String) : ParseVoucherCodeError

    data class TooShort(val value: String) : ParseVoucherCodeError
}
