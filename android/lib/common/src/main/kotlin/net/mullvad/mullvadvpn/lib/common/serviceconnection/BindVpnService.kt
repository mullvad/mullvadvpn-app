package net.mullvad.mullvadvpn.lib.common.serviceconnection

import android.content.Context
import android.content.Context.BIND_AUTO_CREATE
import android.content.Intent
import android.content.ServiceConnection
import android.content.pm.ServiceInfo
import android.os.Build
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS

fun Context.bindVpnService(): ServiceConnection {
    val serviceConnection = EmptyServiceConnection()
    bindVpnService(serviceConnection)
    return serviceConnection
}

fun Context.bindVpnService(serviceConnection: ServiceConnection) {
    val intent = Intent().apply { setClassName(packageName, VPN_SERVICE_CLASS) }

    // We set BIND_AUTO_CREATE so that the service is started if it is not already running
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
        bindService(
            intent,
            serviceConnection,
            ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED or BIND_AUTO_CREATE,
        )
    } else {
        bindService(intent, serviceConnection, BIND_AUTO_CREATE)
    }
}
