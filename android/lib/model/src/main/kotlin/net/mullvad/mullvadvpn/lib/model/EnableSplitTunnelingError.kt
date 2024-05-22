package net.mullvad.mullvadvpn.lib.model

interface EnableSplitTunnelingError {
    data class Unknown(val throwable: Throwable) : EnableSplitTunnelingError
}
