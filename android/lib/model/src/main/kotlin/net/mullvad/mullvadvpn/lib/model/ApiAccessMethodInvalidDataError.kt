package net.mullvad.mullvadvpn.lib.model

data class ApiAccessMethodInvalidDataErrors(val errors: List<InvalidDataError>) {
    inline fun <reified T : InvalidDataError> getErrorOrNull(): T? =
        errors.filterIsInstance<T>().firstOrNull()
}

sealed interface InvalidDataError {
    sealed interface NameError : InvalidDataError {
        data object Required : NameError
    }

    sealed interface ServerIpError : InvalidDataError {
        data object Required : ServerIpError

        data object Invalid : ServerIpError
    }

    sealed interface RemotePortError : InvalidDataError {
        data object Required : RemotePortError

        data object Invalid : RemotePortError
    }

    sealed interface LocalPortError : InvalidDataError {
        data object Required : LocalPortError

        data object Invalid : LocalPortError
    }

    sealed interface UserNameError : InvalidDataError {
        data object Required : UserNameError
    }

    sealed interface PasswordError : InvalidDataError {
        data object Required : PasswordError
    }
}
