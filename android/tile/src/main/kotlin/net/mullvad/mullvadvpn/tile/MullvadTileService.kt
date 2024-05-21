package net.mullvad.mullvadvpn.tile

import android.annotation.SuppressLint
import android.app.PendingIntent
import android.content.Intent
import android.graphics.drawable.Icon
import android.net.VpnService
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import android.util.Log
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.Job
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.setSubtitleIfSupported
import net.mullvad.mullvadvpn.lib.daemon.grpc.GrpcConnectivityState
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.TunnelState
import org.koin.android.ext.android.getKoin

class MullvadTileService : TileService() {
    private var job: Job? = null

    private lateinit var securedIcon: Icon
    private lateinit var unsecuredIcon: Icon

    private lateinit var managementService: ManagementService

    override fun onCreate() {
        Log.d("MullvadTileService", "onCreate")
        securedIcon = Icon.createWithResource(this, R.drawable.small_logo_white)
        unsecuredIcon = Icon.createWithResource(this, R.drawable.small_logo_black)

        with(getKoin()) { managementService = get() }
    }

    override fun onClick() {
        // Workaround for the reported bug: https://issuetracker.google.com/issues/236862865
        suspend fun isUnlockStatusPropagatedWithinTimeout(
            unlockTimeoutMillis: Long,
            unlockCheckDelayMillis: Long
        ): Boolean {
            return withTimeoutOrNull(unlockTimeoutMillis) {
                while (isLocked) {
                    delay(unlockCheckDelayMillis)
                }
                return@withTimeoutOrNull true
            } ?: false
        }

        unlockAndRun {
            runBlocking {
                val isUnlockStatusPropagated =
                    isUnlockStatusPropagatedWithinTimeout(
                        unlockTimeoutMillis = 1000L,
                        unlockCheckDelayMillis = 100L
                    )

                if (isUnlockStatusPropagated) {
                    toggleTunnel()
                } else {
                    Log.e("mullvad", "Unable to toggle tunnel state")
                }
            }
        }
    }

    override fun onStartListening() {
        Log.d("MullvadTileService", "onStartListening")
        job = MainScope().launch { launchListenToTunnelState() }
    }

    override fun onStopListening() {
        Log.d("MullvadTileService", "onStopListening")
        job?.cancel()
    }

    @SuppressLint("StartActivityAndCollapseDeprecated")
    private fun toggleTunnel() {
        val isSetup = VpnService.prepare(applicationContext) == null
        // TODO This logic should be more advanced, we should ensure user has an account setup etc.
        if (!isSetup) {
            Log.d("MullvadTileService", "VPN service not setup, starting main activity")

            val intent =
                Intent().apply {
                    setClassName(applicationContext.packageName, MAIN_ACTIVITY_CLASS)
                    flags =
                        Intent.FLAG_ACTIVITY_CLEAR_TOP or
                            Intent.FLAG_ACTIVITY_SINGLE_TOP or
                            Intent.FLAG_ACTIVITY_NEW_TASK
                    action = Intent.ACTION_MAIN
                }
            startActivityAndCollapseCompat(intent)
            return
        } else {
            Log.d("MullvadTileService", "VPN service is setup")
        }
        val intent =
            Intent().apply {
                setClassName(applicationContext.packageName, VPN_SERVICE_CLASS)
                action =
                    if (qsTile.state == Tile.STATE_INACTIVE) {
                        KEY_CONNECT_ACTION
                    } else {
                        KEY_DISCONNECT_ACTION
                    }
            }

        // Always start as foreground in case tile is out-of-sync.
        startForegroundService(intent)
    }

    @SuppressLint("StartActivityAndCollapseDeprecated")
    private fun MullvadTileService.startActivityAndCollapseCompat(intent: Intent) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            val pendingIntent =
                PendingIntent.getActivity(
                    applicationContext,
                    0,
                    intent,
                    SdkUtils.getSupportedPendingIntentFlags()
                )
            startActivityAndCollapse(pendingIntent)
        } else {
            startActivityAndCollapse(intent)
        }
    }

    @OptIn(FlowPreview::class)
    private suspend fun launchListenToTunnelState() {
        Log.d("MullvadTileService", "launchListenToTunnelState")
        combine(
                managementService.tunnelState.onStart { emit(TunnelState.Disconnected(null)) },
                managementService.connectionState
            ) { tunnelState, connectionState ->
                tunnelState to connectionState
            }
            .debounce(300L)
            .map { (tunnelState, connectionState) -> mapToTileState(tunnelState, connectionState) }
            .collect { updateTileState(it) }
    }

    private fun mapToTileState(
        tunnelState: TunnelState,
        connectionState: GrpcConnectivityState
    ): Int {
        Log.d("MullvadTileService", "mapToTileState: $tunnelState, $connectionState")
        return if (connectionState == GrpcConnectivityState.Ready) {
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
                setSubtitleIfSupported(resources.getText(R.string.secured))
            } else {
                state = Tile.STATE_INACTIVE
                icon = unsecuredIcon
                setSubtitleIfSupported(resources.getText(R.string.unsecured))
            }
            updateTile()
        }
    }
}
