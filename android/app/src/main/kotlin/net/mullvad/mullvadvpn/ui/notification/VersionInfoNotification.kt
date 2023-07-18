package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.constant.BuildTypes
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.util.appendHideNavOnReleaseBuild

class VersionInfoNotification(context: Context) :
    NotificationWithUrl(
        context,
        context.getString(R.string.download_url).appendHideNavOnReleaseBuild()
    ) {
    private val unsupportedVersion = context.getString(R.string.unsupported_version)
    private val updateAvailable = context.getString(R.string.update_available)

    fun updateVersionInfo(versionInfo: VersionInfo) {
        if (versionInfo.isOutdated || !versionInfo.isSupported) {
            if (versionInfo.upgradeVersion != null) {
                message =
                    if (versionInfo.isSupported) {
                        status = StatusLevel.Warning
                        title = updateAvailable
                        context.getString(
                            R.string.update_available_description,
                            versionInfo.upgradeVersion
                        )
                    } else {
                        status = StatusLevel.Error
                        title = unsupportedVersion
                        context.getString(R.string.unsupported_version_description)
                    }
            } else {
                status = StatusLevel.Error
                title = unsupportedVersion
                message = context.getString(R.string.unsupported_version_without_upgrade)
            }

            shouldShow = true
            if (BuildTypes.RELEASE == BuildConfig.BUILD_TYPE) {
                disableExternalLink()
            }
        } else {
            shouldShow = false
        }

        update()
    }
}
