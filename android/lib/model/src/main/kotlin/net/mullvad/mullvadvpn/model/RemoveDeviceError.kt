package net.mullvad.mullvadvpn.model

sealed interface RemoveDeviceError {
    data object NotFound : RemoveDeviceError

    data object RpcError : RemoveDeviceError

    data class Unknown(val throwable: Throwable) : RemoveDeviceError
}
