package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache

fun AppVersionInfoCache.appVersionCallbackFlow() = callbackFlow {
    this@appVersionCallbackFlow.onUpdate = {
        trySend(
            VersionInfo(
                currentVersion = this@appVersionCallbackFlow.version,
                upgradeVersion = this@appVersionCallbackFlow.upgradeVersion,
                isOutdated = this@appVersionCallbackFlow.isOutdated,
                isSupported = this@appVersionCallbackFlow.isSupported,
            )
        )
    }
    awaitClose {
        onUpdate = null
    }
}
