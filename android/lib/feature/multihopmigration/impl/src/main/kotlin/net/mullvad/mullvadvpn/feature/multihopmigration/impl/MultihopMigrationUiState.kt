package net.mullvad.mullvadvpn.feature.multihopmigration.impl

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

data class MultihopMigrationUiState(
    val multihopMigrationPages: List<MultihopMigrationPage>,
    val currentPageIndex: Int,
) {
    val size
        get() = multihopMigrationPages.size
}

@Parcelize
sealed interface MultihopMigrationPage : Parcelable {
    data class NewMultihopMode(val multihopMigrationState: MultihopMigrationState) :
        MultihopMigrationPage

    data object DirectOnlyRemoved : MultihopMigrationPage

    data object SeparateFilters : MultihopMigrationPage

    data object SuggestedMultihopEntry : MultihopMigrationPage

    data object SuggestedAction : MultihopMigrationPage
}

enum class MultihopMigrationState {
    ON_TO_ALWAYS,
    OFF_TO_NEVER,
    OFF_TO_WHEN_NEEDED,
    OFF_TO_ALWAYS,
}
