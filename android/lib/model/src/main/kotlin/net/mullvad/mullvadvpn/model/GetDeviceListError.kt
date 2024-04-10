package net.mullvad.mullvadvpn.model

sealed interface GetDeviceListError {
    data class Unknown(val error: Throwable): GetDeviceListError
}