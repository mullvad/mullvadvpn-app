package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository

class NewChangelogNotificationUseCase(
    private val userPreferencesRepository: UserPreferencesRepository,
    private val changelogRepository: ChangelogRepository,
    private val buildVersion: BuildVersion,
) {
    operator fun invoke() =
        combine(
                userPreferencesRepository.preferencesFlow,
                changelogRepository.showNewChangelogNotification,
            ) { preferences, showChangelog ->
                if (
                    buildVersion.code > preferences.versionCodeForLatestShownChangelogNotification
                ) {
                    changelogRepository.setShowNewChangelogNotification()
                    userPreferencesRepository.setHasDisplayedChangelogNotification()
                }
                if (showChangelog) {
                    InAppNotification.NewVersionChangelog
                } else null
            }
            .map(::listOfNotNull)
            .distinctUntilChanged()
}
