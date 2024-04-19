package net.mullvad.mullvadvpn.model

interface AddSplitTunnelingAppError {
    data class Unknown(val throwable: Throwable) : AddSplitTunnelingAppError
}
