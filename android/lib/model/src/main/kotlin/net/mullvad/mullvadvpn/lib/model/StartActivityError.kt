package net.mullvad.mullvadvpn.lib.model

sealed interface StartActivityError {
    object ActivityNotFound : StartActivityError
}
