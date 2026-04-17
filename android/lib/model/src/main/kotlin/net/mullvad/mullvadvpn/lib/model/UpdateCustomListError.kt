package net.mullvad.mullvadvpn.lib.model

sealed interface UpdateCustomListNameError {
    companion object {
        fun from(error: UpdateCustomListError): UpdateCustomListNameError =
            when (error) {
                is NameAlreadyExists -> error
                is NameIsEmpty -> error
                is UnknownCustomListError -> error
            }
    }
}

sealed interface UpdateCustomListLocationsError {
    companion object {
        fun from(error: UpdateCustomListError): UpdateCustomListLocationsError =
            when (error) {
                is NameAlreadyExists -> error("Not supported error")
                is NameIsEmpty -> error("Not supported error")
                is UnknownCustomListError -> error
            }
    }
}

sealed interface UpdateCustomListError

data class NameAlreadyExists(val name: CustomListName) :
    UpdateCustomListError, UpdateCustomListNameError

data class NameIsEmpty(val name: CustomListName) :
    UpdateCustomListError, UpdateCustomListNameError, CreateCustomListError

data class UnknownCustomListError(val throwable: Throwable) :
    UpdateCustomListError,
    UpdateCustomListNameError,
    UpdateCustomListLocationsError,
    CreateCustomListError,
    DeleteCustomListError

data class GetCustomListError(val id: CustomListId) :
    UpdateCustomListLocationsError, UpdateCustomListNameError
