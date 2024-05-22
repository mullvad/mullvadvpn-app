package net.mullvad.mullvadvpn.lib.shared

import android.content.Context
import android.net.VpnService
import net.mullvad.mullvadvpn.lib.common.util.getAlwaysOnVpnAppName

class VpnPermissionRepository(private val applicationContext: Context) {
    fun hasVpnPermission(): Boolean = VpnService.prepare(applicationContext) == null

    fun getAlwaysOnVpnAppName() = applicationContext.getAlwaysOnVpnAppName()
}
