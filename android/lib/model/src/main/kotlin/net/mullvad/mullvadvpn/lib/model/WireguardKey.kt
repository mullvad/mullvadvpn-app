package net.mullvad.mullvadvpn.lib.model

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import kotlin.io.encoding.Base64

@JvmInline
value class WireguardKey private constructor(val value: String) {

    companion object {
        const val KEY_BASE_64_LENGTH = 44

        fun from(value: String): Either<KeyParseError, WireguardKey> = either {
            ensure(value.length == KEY_BASE_64_LENGTH) {
                KeyParseError.InvalidLength(value.length, KEY_BASE_64_LENGTH)
            }

            ensure(value.last() == '=') { KeyParseError.InvalidEnding }

            Either.catch({ Base64.decode(value) })
                .mapLeft { KeyParseError.InvalidBase64(it) }
                .onLeft { raise(it) }

            WireguardKey(value)
        }
    }
}
