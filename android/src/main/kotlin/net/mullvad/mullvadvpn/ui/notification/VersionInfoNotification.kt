package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache

class VersionInfoNotification(
    context: Context,
    private val versionInfoCache: AppVersionInfoCache
) : NotificationWithUrl(context, R.string.download_url) {
    private val unsupportedVersion = context.getString(R.string.unsupported_version)
    private val updateAvailable = context.getString(R.string.update_available)

    override fun onResume() {
        versionInfoCache.onUpdate = {
            jobTracker.newUiJob("updateVersionInfo") {
                updateVersionInfo(
                    versionInfoCache.isOutdated,
                    versionInfoCache.isSupported,
                    versionInfoCache.upgradeVersion
                )
            }
        }
    }

    override fun onPause() {
        versionInfoCache.onUpdate = null
    }

    private fun updateVersionInfo(isOutdated: Boolean, isSupported: Boolean, upgrade: String?) {
        if (isOutdated || !isSupported) {
            val template: Int

            if (isSupported) {
                status = StatusLevel.Warning
                title = updateAvailable
                template = R.string.update_available_description
            } else {
                status = StatusLevel.Error
                title = unsupportedVersion
                template = R.string.unsupported_version_description
            }

            message = context.getString(template, upgrade)

            shouldShow = true
        } else {
            shouldShow = false
        }

        update()
    }
}
