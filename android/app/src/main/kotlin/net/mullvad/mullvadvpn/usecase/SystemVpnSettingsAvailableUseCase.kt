package net.mullvad.mullvadvpn.usecase

import android.content.Context
import android.content.Intent

class SystemVpnSettingsAvailableUseCase(val context: Context) {
    operator fun invoke(): Boolean =
        Intent("android.net.vpn.SETTINGS").resolveActivity(context.packageManager) != null
}
