package net.mullvad.mullvadvpn.lib.model

interface RemoveSplitTunnelingAppError {
    data class Unknown(val throwable: Throwable) : RemoveSplitTunnelingAppError
}
