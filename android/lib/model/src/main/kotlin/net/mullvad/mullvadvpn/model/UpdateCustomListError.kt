package net.mullvad.mullvadvpn.model

sealed interface UpdateCustomListError : ModifyCustomListError {
    data class NameAlreadyExists(val name: String) : UpdateCustomListError

    data class Unknown(val throwable: Throwable) : UpdateCustomListError
}

sealed interface ModifyCustomListError

data object GetCustomListError : ModifyCustomListError
