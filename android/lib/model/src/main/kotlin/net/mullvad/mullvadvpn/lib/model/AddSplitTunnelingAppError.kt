package net.mullvad.mullvadvpn.lib.model

interface AddSplitTunnelingAppError {
    data class Unknown(val throwable: Throwable) : AddSplitTunnelingAppError
}
