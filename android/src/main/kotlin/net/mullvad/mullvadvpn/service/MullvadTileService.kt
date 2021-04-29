package net.mullvad.mullvadvpn.service

import android.content.ComponentName
import android.content.Intent
import android.graphics.drawable.Icon
import android.os.Build
import android.os.IBinder
import android.os.Messenger
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnection
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class MullvadTileService : TileService() {
    private val serviceConnectionManager = object : android.content.ServiceConnection {
        override fun onServiceConnected(className: ComponentName, binder: IBinder) {
            serviceConnection = ServiceConnection(Messenger(binder))
        }

        override fun onServiceDisconnected(className: ComponentName) {
            serviceConnection = null
        }
    }

    private var serviceConnection by observable<ServiceConnection?>(
        null
    ) { _, oldConnection, newConnection ->
        oldConnection?.onDestroy()

        newConnection?.connectionProxy?.run {
            onStateChange.subscribe(this@MullvadTileService, ::updateTunnelState)
        }
    }

    private var secured by observable(false) { _, wasSecured, isSecured ->
        if (wasSecured != isSecured) {
            updateTileState()
        }
    }

    private lateinit var securedIcon: Icon
    private lateinit var unsecuredIcon: Icon

    override fun onCreate() {
        super.onCreate()

        securedIcon = Icon.createWithResource(this, R.drawable.small_logo_white)
        unsecuredIcon = Icon.createWithResource(this, R.drawable.small_logo_black)
    }

    override fun onStartListening() {
        super.onStartListening()

        val intent = Intent(this, MullvadVpnService::class.java)

        bindService(intent, serviceConnectionManager, BIND_IMPORTANT)

        updateTileState()
    }

    override fun onClick() {
        super.onClick()

        val intent = Intent(this, MullvadVpnService::class.java)

        if (secured) {
            intent.action = MullvadVpnService.KEY_DISCONNECT_ACTION
        } else {
            intent.action = MullvadVpnService.KEY_CONNECT_ACTION
        }

        if (Build.VERSION.SDK_INT >= 26) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }

    override fun onStopListening() {
        unbindService(serviceConnectionManager)
        serviceConnection = null

        super.onStopListening()
    }

    private fun updateTunnelState(tunnelState: TunnelState) {
        secured = when (tunnelState) {
            is TunnelState.Disconnected -> false
            is TunnelState.Connecting -> true
            is TunnelState.Connected -> true
            is TunnelState.Disconnecting -> {
                tunnelState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect
            }
            is TunnelState.Error -> tunnelState.errorState.isBlocking
        }
    }

    private fun updateTileState() {
        qsTile?.apply {
            if (secured) {
                state = Tile.STATE_ACTIVE
                icon = securedIcon
            } else {
                state = Tile.STATE_INACTIVE
                icon = unsecuredIcon
            }

            updateTile()
        }
    }
}
