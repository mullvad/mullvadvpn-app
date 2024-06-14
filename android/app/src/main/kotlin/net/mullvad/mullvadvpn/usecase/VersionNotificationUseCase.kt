package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository

class VersionNotificationUseCase(
    private val appVersionInfoRepository: AppVersionInfoRepository,
    private val isVersionInfoNotificationEnabled: Boolean,
) {

    operator fun invoke() =
        appVersionInfoRepository
            .versionInfo()
            .map { versionInfo ->
                listOfNotNull(
                    unsupportedVersionNotification(versionInfo),
                    updateAvailableNotification(versionInfo)
                )
            }
            .distinctUntilChanged()

    private fun updateAvailableNotification(versionInfo: VersionInfo): InAppNotification? {
        if (!isVersionInfoNotificationEnabled) {
            return null
        }

        return if (versionInfo.isUpdateAvailable) {
            InAppNotification.UpdateAvailable(versionInfo)
        } else null
    }

    private fun unsupportedVersionNotification(versionInfo: VersionInfo): InAppNotification? {
        if (!isVersionInfoNotificationEnabled) {
            return null
        }

        return if (!versionInfo.isSupported) {
            InAppNotification.UnsupportedVersion(versionInfo)
        } else null
    }
}
