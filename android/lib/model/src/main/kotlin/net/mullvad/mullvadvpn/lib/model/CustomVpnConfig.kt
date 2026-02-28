package net.mullvad.mullvadvpn.lib.model

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import java.net.InetAddress
import java.net.InetSocketAddress
import java.time.Instant
import kotlin.io.encoding.Base64

sealed interface SetCustomVpnConfigError {
    object Unknown : SetCustomVpnConfigError
}

sealed interface GetCustomVpnConfigError {
    object Unknown : GetCustomVpnConfigError
}

data class CustomVpnConfig(val tunnelConfig: TunnelConfig, val peerConfig: PeerConfig)

data class TunnelConfig(val privateKey: WireguardKey, val tunnelIp: InetAddress)

data class PeerConfig(
    val publicKey: WireguardKey,
    val endpoint: InetSocketAddress,
    val allowedIp: String,
)

data class TunnelStats(val rx: Long = 0, val tx: Long = 0, val lastHandshake: Instant? = null)

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

sealed interface KeyParseError {
    data class InvalidLength(val actualLenght: Int, val expectedLength: Int) : KeyParseError

    data object InvalidEnding : KeyParseError

    data class InvalidBase64(val throwable: Throwable) : KeyParseError
}
