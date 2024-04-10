package net.mullvad.mullvadvpn.model

interface ListDevicesError {
    data class Unknown(val throwable: Throwable) : ListDevicesError
}
