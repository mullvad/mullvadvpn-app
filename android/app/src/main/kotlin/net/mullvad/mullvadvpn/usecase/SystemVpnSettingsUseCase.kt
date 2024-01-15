package net.mullvad.mullvadvpn.usecase

import android.content.Context
import android.content.Intent

class SystemVpnSettingsUseCase(val context: Context) {
    fun systemVpnSettingsAvailable(): Boolean =
        Intent("android.net.vpn.SETTINGS").resolveActivity(context.packageManager) != null
}
