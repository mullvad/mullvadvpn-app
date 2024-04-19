package net.mullvad.mullvadvpn.model

interface RemoveSplitTunnelingAppError {
    data class Unknown(val throwable: Throwable) : RemoveSplitTunnelingAppError
}
