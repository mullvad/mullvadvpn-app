package net.mullvad.mullvadvpn.model

interface SetAllowLanError {
    data class Unknown(val throwable: Throwable) : SetAllowLanError
}
