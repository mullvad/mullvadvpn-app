package net.mullvad.mullvadvpn.lib.model

data class SplitFilterMigration(
    val multihopMigrationState: MultihopMigrationState,
    val filtersSet: Boolean,
    val daitaMigration: PreviousDaitaState,
)
