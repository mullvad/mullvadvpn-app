package net.mullvad.mullvadvpn.usecase.inappnotification

import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.repository.ChangelogRepository

class NewChangelogNotificationUseCase(private val changelogRepository: ChangelogRepository) :
    InAppNotificationUseCase {
    override operator fun invoke() =
        changelogRepository.hasUnreadChangelog
            .map { if (it) InAppNotification.NewVersionChangelog else null }
            .distinctUntilChanged()
}
