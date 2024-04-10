package net.mullvad.mullvadvpn.model

sealed interface DeleteDeviceError {
    data class Unknown(val error: Throwable) : DeleteDeviceError
}
