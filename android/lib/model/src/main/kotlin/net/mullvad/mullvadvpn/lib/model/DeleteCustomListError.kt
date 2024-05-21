package net.mullvad.mullvadvpn.lib.model

sealed interface DeleteCustomListError {
    data class Unknown(val throwable: Throwable) : DeleteCustomListError
}
