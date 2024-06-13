package net.mullvad.mullvadvpn.lib.model

sealed interface GetApiAccessMethodError : UpdateApiAccessMethodError {
    data object NotFound : GetApiAccessMethodError
}
