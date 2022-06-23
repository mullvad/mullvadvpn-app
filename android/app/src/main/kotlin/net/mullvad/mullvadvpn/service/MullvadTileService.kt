package net.mullvad.mullvadvpn.service

import android.content.Intent
import android.graphics.drawable.Icon
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.Job
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ipc.ServiceConnection
import net.mullvad.mullvadvpn.model.ServiceResult
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class MullvadTileService : TileService() {
    private val scope = MainScope()
    private var listenerJob: Job? = null

    private lateinit var securedIcon: Icon
    private lateinit var unsecuredIcon: Icon

    override fun onCreate() {
        super.onCreate()

        securedIcon = Icon.createWithResource(this, R.drawable.small_logo_white)
        unsecuredIcon = Icon.createWithResource(this, R.drawable.small_logo_black)
    }

    override fun onClick() {
        val intent = Intent(this, MullvadVpnService::class.java).apply {
            action = if (qsTile.state == Tile.STATE_ACTIVE) {
                MullvadVpnService.KEY_DISCONNECT_ACTION
            } else {
                MullvadVpnService.KEY_CONNECT_ACTION
            }
        }
        startForegroundService(intent)
    }

    override fun onStartListening() {
        listenerJob = scope.launch { listenToTunnelState() }
    }

    override fun onStopListening() {
        listenerJob?.cancel()
    }

    @OptIn(FlowPreview::class)
    private suspend fun listenToTunnelState() {
        ServiceConnection(this@MullvadTileService, scope)
            .tunnelState
            .debounce(300L)
            .map { (tunnelState, connectionState) -> mapToTileState(tunnelState, connectionState) }
            .collect {
                updateTileState(it)
            }
    }

    private fun mapToTileState(
        tunnelState: TunnelState,
        connectionState: ServiceResult.ConnectionState
    ): Int {
        return if (connectionState == ServiceResult.ConnectionState.CONNECTED) {
            when (tunnelState) {
                is TunnelState.Disconnected -> Tile.STATE_INACTIVE
                is TunnelState.Connecting -> Tile.STATE_ACTIVE
                is TunnelState.Connected -> Tile.STATE_ACTIVE
                is TunnelState.Disconnecting -> {
                    if (tunnelState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect) {
                        Tile.STATE_ACTIVE
                    } else {
                        Tile.STATE_INACTIVE
                    }
                }
                is TunnelState.Error -> {
                    if (tunnelState.errorState.isBlocking) {
                        Tile.STATE_ACTIVE
                    } else {
                        Tile.STATE_INACTIVE
                    }
                }
            }
        } else {
            Tile.STATE_INACTIVE
        }
    }

    private fun updateTileState(newState: Int) {
        qsTile?.apply {
            if (newState == Tile.STATE_ACTIVE) {
                state = Tile.STATE_ACTIVE
                icon = securedIcon

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    subtitle = resources.getText(R.string.secured)
                }
            } else {
                state = Tile.STATE_INACTIVE
                icon = unsecuredIcon

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    subtitle = resources.getText(R.string.unsecured)
                }
            }
            updateTile()
        }
    }
}
