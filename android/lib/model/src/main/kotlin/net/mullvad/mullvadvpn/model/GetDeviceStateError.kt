package net.mullvad.mullvadvpn.model

sealed interface GetDeviceStateError {
    data class Unknown(val error: Throwable) : GetDeviceStateError
}
