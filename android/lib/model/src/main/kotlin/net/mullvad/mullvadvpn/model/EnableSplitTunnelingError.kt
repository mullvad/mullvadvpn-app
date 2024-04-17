package net.mullvad.mullvadvpn.model

interface EnableSplitTunnelingError {
    data class Unknown(val throwable: Throwable) : EnableSplitTunnelingError
}
