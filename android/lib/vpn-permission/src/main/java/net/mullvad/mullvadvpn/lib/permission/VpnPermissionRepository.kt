package net.mullvad.mullvadvpn.lib.permission

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.receiveAsFlow
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS
import net.mullvad.mullvadvpn.lib.common.util.firstOrNullWithTimeout

class VpnPermissionRepository(private val applicationContext: Context) {
    fun hasVpnPermission(): Boolean = VpnService.prepare(applicationContext) == null
}
