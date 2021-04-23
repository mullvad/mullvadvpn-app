package net.mullvad.mullvadvpn.service

import android.content.Intent
import android.graphics.drawable.Icon
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.tunnelstate.TunnelStateListener
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class MullvadTileService : TileService() {
    private var secured by observable(false) { _, wasSecured, isSecured ->
        if (wasSecured != isSecured) {
            updateTileState()
        }
    }

    private lateinit var listener: TunnelStateListener
    private lateinit var securedIcon: Icon
    private lateinit var unsecuredIcon: Icon

    override fun onCreate() {
        super.onCreate()

        listener = TunnelStateListener(this)
        securedIcon = Icon.createWithResource(this, R.drawable.small_logo_white)
        unsecuredIcon = Icon.createWithResource(this, R.drawable.small_logo_black)
    }

    override fun onStartListening() {
        super.onStartListening()

        listener.onStateChange = ::updateTunnelState

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
        listener.onStateChange = null

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
