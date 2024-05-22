package net.mullvad.mullvadvpn.lib.model

sealed interface GetDeviceStateError {
    data class Unknown(val error: Throwable) : GetDeviceStateError
}
