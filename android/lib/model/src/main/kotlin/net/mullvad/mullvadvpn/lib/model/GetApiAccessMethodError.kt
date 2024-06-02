package net.mullvad.mullvadvpn.lib.model

sealed interface GetApiAccessMethodError {
    data object NotFound : GetApiAccessMethodError
}
