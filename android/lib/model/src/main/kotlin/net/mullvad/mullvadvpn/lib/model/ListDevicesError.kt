package net.mullvad.mullvadvpn.lib.model

interface ListDevicesError {
    data class Unknown(val throwable: Throwable) :
        net.mullvad.mullvadvpn.lib.model.ListDevicesError
}
