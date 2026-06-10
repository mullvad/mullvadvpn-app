package net.mullvad.mullvadvpn.feature.multihopmigration.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider

class MultihopMigrationUiStatePreviewParameterProvider :
    PreviewParameterProvider<MultihopMigrationUiState> {
    override val values: Sequence<MultihopMigrationUiState> =
        (scenario1b +
                scenario2 +
                scenario3a +
                scenario3b +
                scenario4a +
                scenario4b +
                scenario5a +
                scenario5b +
                scenario6a +
                scenario6b +
                scenario7a +
                scenario7b +
                catchAllError)
            .asSequence()

    companion object {
        // Scenarios according to the design spec (scenario 1a does not show the guide):
        // 1b
        private val scenario1b =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER),
                    MultihopMigrationPage.SeparateFilters,
                    MultihopMigrationPage.SuggestedAction,
                )
                .toUiState()
        // 2
        private val scenario2 =
            listOf(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_WHEN_NEEDED))
                .toUiState()
        // 3a
        private val scenario3a =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER),
                    MultihopMigrationPage.SeparateFilters,
                    MultihopMigrationPage.SuggestedAction,
                )
                .toUiState()
        // 3b
        private val scenario3b =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_ALWAYS),
                    MultihopMigrationPage.SeparateFilters,
                    MultihopMigrationPage.SuggestedMultihopEntry,
                )
                .toUiState()
        // 4a
        private val scenario4a =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER),
                    MultihopMigrationPage.DirectOnlyRemoved,
                )
                .toUiState()
        // 4b
        private val scenario4b =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER),
                    MultihopMigrationPage.DirectOnlyRemoved,
                )
                .toUiState()
        // 5a
        private val scenario5a =
            listOf(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
                .toUiState()
        // 5b
        private val scenario5b =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS),
                    MultihopMigrationPage.SeparateFilters,
                )
                .toUiState()
        // 6a
        private val scenario6a =
            listOf(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
                .toUiState()
        // 6b
        private val scenario6b =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS),
                    MultihopMigrationPage.SeparateFilters,
                    MultihopMigrationPage.SuggestedMultihopEntry,
                )
                .toUiState()
        // 7a
        private val scenario7a =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS),
                    MultihopMigrationPage.DirectOnlyRemoved,
                )
                .toUiState()
        // 7b
        private val scenario7b =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS),
                    MultihopMigrationPage.DirectOnlyRemoved,
                    MultihopMigrationPage.SeparateFilters,
                    MultihopMigrationPage.SuggestedMultihopEntry,
                )
                .toUiState()
        // Catch-all-error
        // Catch all errors can show any pages but will always show the suggested action page at the
        // end. The other pages are just as an example.
        private val catchAllError =
            listOf(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS),
                    MultihopMigrationPage.SeparateFilters,
                    MultihopMigrationPage.SuggestedAction,
                )
                .toUiState()

        private fun List<MultihopMigrationPage>.toUiState(): List<MultihopMigrationUiState> =
            buildList {
                repeat(this@toUiState.size) { index ->
                    add(
                        MultihopMigrationUiState(
                            multihopMigrationPages = this@toUiState,
                            currentPageIndex = index,
                        )
                    )
                }
            }
    }
}
