package net.mullvad.mullvadvpn.lib.model

sealed interface DeleteDeviceError {
    data class Unknown(val error: Throwable) : DeleteDeviceError
}
