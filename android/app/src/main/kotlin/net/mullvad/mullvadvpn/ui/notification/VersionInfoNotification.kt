package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.common.constant.BuildTypes
import net.mullvad.mullvadvpn.lib.common.util.appendHideNavOnReleaseBuild
import net.mullvad.mullvadvpn.ui.VersionInfo

class VersionInfoNotification(val isEnabled: Boolean, context: Context) :
    NotificationWithUrl(
        context,
        context.getString(R.string.download_url).appendHideNavOnReleaseBuild()
    ) {
    private val unsupportedVersion = context.getString(R.string.unsupported_version)
    private val updateAvailable = context.getString(R.string.update_available)

    fun updateVersionInfo(versionInfo: VersionInfo) {
        val shouldShowNotification =
            isEnabled && (versionInfo.isOutdated || !versionInfo.isSupported)

        if (shouldShowNotification) {
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
            if (BuildConfig.BUILD_TYPE == BuildTypes.RELEASE) {
                disableExternalLink()
            }
        } else {
            shouldShow = false
        }

        update()
    }
}
