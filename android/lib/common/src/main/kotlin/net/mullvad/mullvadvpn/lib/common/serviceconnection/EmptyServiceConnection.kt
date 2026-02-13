package net.mullvad.mullvadvpn.lib.common.serviceconnection

import android.content.ComponentName
import android.content.ServiceConnection
import android.os.IBinder

class EmptyServiceConnection : ServiceConnection {
    @Suppress("EmptyFunctionBlock")
    override fun onServiceConnected(name: ComponentName?, service: IBinder?) {}

    @Suppress("EmptyFunctionBlock") override fun onServiceDisconnected(name: ComponentName?) {}

    override fun onNullBinding(name: ComponentName?) {
        error("Received onNullBinding")
    }
}
