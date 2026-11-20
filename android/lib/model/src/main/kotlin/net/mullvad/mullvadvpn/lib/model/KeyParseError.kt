package net.mullvad.mullvadvpn.lib.model

sealed interface KeyParseError {
    data class InvalidLength(val actualLength: Int, val expectedLength: Int) : KeyParseError

    data object InvalidEnding : KeyParseError

    data class InvalidBase64(val throwable: Throwable) : KeyParseError
}
