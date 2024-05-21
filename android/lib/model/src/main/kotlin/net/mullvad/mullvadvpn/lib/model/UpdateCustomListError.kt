package net.mullvad.mullvadvpn.lib.model

sealed interface UpdateCustomListError : ModifyCustomListError {
    data class NameAlreadyExists(val name: String) : UpdateCustomListError

    data class Unknown(val throwable: Throwable) : UpdateCustomListError
}

sealed interface ModifyCustomListError

data class GetCustomListError(val id: net.mullvad.mullvadvpn.lib.model.CustomListId) : ModifyCustomListError
