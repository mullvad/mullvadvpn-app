package net.mullvad.mullvadvpn.model

sealed interface CreateCustomListError {
    data object CustomListAlreadyExists : CreateCustomListError

    data class Unknown(val throwable: Throwable) : CreateCustomListError
}
