package net.mullvad.mullvadvpn.feature.multihopmigration.impl

sealed interface MultihopMigrationScreenSideEffect {
    data object CloseScreen : MultihopMigrationScreenSideEffect

    data object GenericError : MultihopMigrationScreenSideEffect
}
