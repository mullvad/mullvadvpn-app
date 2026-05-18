package net.mullvad.mullvadvpn.lib.model

sealed interface ClearMigrationMessageError {
    data class Unknown(val throwable: Throwable) : ClearMigrationMessageError
}
