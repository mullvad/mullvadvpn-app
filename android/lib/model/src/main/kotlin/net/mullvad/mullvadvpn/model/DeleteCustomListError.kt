package net.mullvad.mullvadvpn.model

sealed interface DeleteCustomListError {
    data class Unknown(val throwable: Throwable) : DeleteCustomListError
}

