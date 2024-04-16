package net.mullvad.mullvadvpn.repository

import android.content.Context
import android.net.VpnService

class VpnPermissionRepository(private val applicationContext: Context) {
    fun hasVpnPermission(): Boolean = VpnService.prepare(applicationContext) == null
}
