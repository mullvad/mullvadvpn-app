package net.mullvad.mullvadvpn.lib.model

sealed interface GetMigrationEventError {
    data class Unknown(val throwable: Throwable) : GetMigrationEventError
}
