package net.mullvad.mullvadvpn.lib.model

// Get followed by and update
sealed interface PartialUpdateCustomListError

sealed interface UpdateCustomListError : PartialUpdateCustomListError {
    data class NameAlreadyExists(val name: String) : UpdateCustomListError

    data class Unknown(val throwable: Throwable) : UpdateCustomListError
}

data class GetCustomListError(val id: CustomListId) : PartialUpdateCustomListError
