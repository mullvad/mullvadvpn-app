package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache

class VersionNotificationUseCase(
    private val appVersionInfoCache: AppVersionInfoCache,
    private val isVersionInfoNotificationEnabled: Boolean,
) {

    fun notifications() =
        appVersionInfoCache
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

        return if (versionInfo.isOutdated) {
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
