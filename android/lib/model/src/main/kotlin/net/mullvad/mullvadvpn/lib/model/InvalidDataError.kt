package net.mullvad.mullvadvpn.lib.model

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

        data class Invalid(val portError: ParsePortError) : RemotePortError
    }

    sealed interface UserNameError : InvalidDataError {
        data object Required : UserNameError
    }

    sealed interface PasswordError : InvalidDataError {
        data object Required : PasswordError
    }
}
