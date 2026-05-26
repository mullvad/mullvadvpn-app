package net.mullvad.mullvadvpn.feature.multihopmigration.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.SplitFilterMigration

@Parcelize
data class MultihopMigrationNavKey(
    val errorFallback: Boolean,
    val migration: SplitFilterMigration,
) : NavKey2
