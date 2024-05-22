package net.mullvad.mullvadvpn.lib.model

sealed interface CreateCustomListError {
    data object CustomListAlreadyExists : CreateCustomListError

    data class Unknown(val throwable: Throwable) : CreateCustomListError
}
