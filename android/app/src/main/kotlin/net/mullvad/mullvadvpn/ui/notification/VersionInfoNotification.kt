package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.VersionInfo

class VersionInfoNotification(context: Context) :
    NotificationWithUrl(context, R.string.download_url) {
    private val unsupportedVersion = context.getString(R.string.unsupported_version)
    private val updateAvailable = context.getString(R.string.update_available)

    fun updateVersionInfo(versionInfo: VersionInfo) {
        if (versionInfo.isOutdated || !versionInfo.isSupported) {
            if (versionInfo.upgradeVersion != null) {
                val template: Int

                if (versionInfo.isSupported) {
                    status = StatusLevel.Warning
                    title = updateAvailable
                    template = R.string.update_available_description
                } else {
                    status = StatusLevel.Error
                    title = unsupportedVersion
                    template = R.string.unsupported_version_description
                }

                message = context.getString(template, versionInfo.upgradeVersion)
            } else {
                status = StatusLevel.Error
                title = unsupportedVersion
                message = context.getString(R.string.unsupported_version_without_upgrade)
            }

            shouldShow = true
        } else {
            shouldShow = false
        }

        update()
    }
}
