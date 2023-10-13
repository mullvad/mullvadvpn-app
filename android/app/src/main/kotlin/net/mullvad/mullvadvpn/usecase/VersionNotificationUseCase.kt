package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault

class VersionNotificationUseCase(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val isVersionInfoNotificationEnabled: Boolean,
) {

    fun notifications() =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf(emptyList())) {
                it.container.appVersionInfoCache.appVersionCallbackFlow().map { versionInfo ->
                    listOfNotNull(
                        unsupportedVersionNotification(versionInfo),
                        updateAvailableNotification(versionInfo)
                    )
                }
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
