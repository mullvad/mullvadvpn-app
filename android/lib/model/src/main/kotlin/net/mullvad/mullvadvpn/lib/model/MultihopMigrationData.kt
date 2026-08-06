package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class MultihopMigrationData(
    val splitFilterMigration: SplitFilterMigration,
    val userBlocked: Boolean,
) : Parcelable
