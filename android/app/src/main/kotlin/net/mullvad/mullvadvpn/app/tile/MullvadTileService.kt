package net.mullvad.mullvadvpn.app.tile

import android.annotation.SuppressLint
import android.app.PendingIntent
import android.content.Intent
import android.graphics.drawable.Icon
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import co.touchlab.kermit.Logger
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.Job
import kotlinx.coroutines.MainScope
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
import net.mullvad.mullvadvpn.lib.common.util.getSupportedPendingIntentFlags
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.grpc.GrpcConnectivityState
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.ui.resource.R
import org.koin.android.ext.android.get

class MullvadTileService : TileService() {
    private var job: Job? = null

    private lateinit var securedIcon: Icon
    private lateinit var unsecuredIcon: Icon

    private val connectionProxy = get<ConnectionProxy>()
    private val managementService = get<ManagementService>()

    override fun onCreate() {
        securedIcon = Icon.createWithResource(this, R.drawable.small_logo_white)
        unsecuredIcon = Icon.createWithResource(this, R.drawable.small_logo_black)
    }

    override fun onClick() {
        // Workaround for the reported bug: https://issuetracker.google.com/issues/236862865
        suspend fun isUnlockStatusPropagatedWithinTimeout(
            unlockTimeout: Duration,
            unlockCheckDelay: Duration,
        ): Boolean {
            return withTimeoutOrNull(unlockTimeout) {
                while (isLocked) {
                    delay(unlockCheckDelay)
                }
                return@withTimeoutOrNull true
            } ?: false
        }

        unlockAndRun {
            runBlocking {
                val isUnlockStatusPropagated =
                    isUnlockStatusPropagatedWithinTimeout(
                        unlockTimeout = 1.seconds,
                        unlockCheckDelay = 100.milliseconds,
                    )

                if (isUnlockStatusPropagated) {
                    toggleTunnel()
                } else {
                    Logger.e("Unable to toggle tunnel state")
                }
            }
        }
    }

    override fun onStartListening() {
        job = MainScope().launch { launchListenToTunnelState() }
    }

    override fun onStopListening() {
        job?.cancel()
    }

    @SuppressLint("StartActivityAndCollapseDeprecated")
    private fun toggleTunnel() {
        val isSetup = isSetup()
        if (isSetup) {
            Logger.i("TileService: VPN service is setup")

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

            // Always start as foreground, e.g. if app is dead we won't be allowed to start if not
            // in foreground.
            startForegroundService(intent)
        } else {
            Logger.i("TileService: VPN service not setup, starting main activity")

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
        }
    }

    @SuppressLint("StartActivityAndCollapseDeprecated")
    private fun MullvadTileService.startActivityAndCollapseCompat(intent: Intent) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            val pendingIntent =
                PendingIntent.getActivity(
                    applicationContext,
                    0,
                    intent,
                    getSupportedPendingIntentFlags(),
                )
            startActivityAndCollapse(pendingIntent)
        } else {
            @Suppress("DEPRECATION") startActivityAndCollapse(intent)
        }
    }

    @OptIn(FlowPreview::class)
    private suspend fun launchListenToTunnelState() {
        combine(
                connectionProxy.tunnelState.onStart { emit(TunnelState.Disconnected(null)) },
                managementService.connectionState,
            ) { tunnelState, connectionState ->
                tunnelState to connectionState
            }
            .debounce(TUNNEL_STATE_DEBOUNCE)
            .map { (tunnelState, connectionState) ->
                mapToTileState(
                    tunnelState = tunnelState,
                    connectionState = connectionState,
                    isSetup = isSetup(),
                )
            }
            .collect { updateTileState(it) }
    }

    private fun mapToTileState(
        tunnelState: TunnelState,
        connectionState: GrpcConnectivityState,
        isSetup: Boolean,
    ): TileState {
        if (!isSetup) {
            return TileState.Unavailable(
                subtitle = resources.getString(R.string.disconnected_vpn_permission_error)
            )
        }

        return if (connectionState == GrpcConnectivityState.Ready) {
            when (tunnelState) {
                is TunnelState.Disconnected ->
                    TileState.Inactive(subtitle = resources.getString(R.string.disconnected))
                is TunnelState.Connecting ->
                    TileState.Active(subtitle = resources.getString(R.string.connecting))
                is TunnelState.Connected ->
                    TileState.Active(subtitle = resources.getString(R.string.connected))
                is TunnelState.Disconnecting -> {
                    if (tunnelState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect) {
                        TileState.Active(subtitle = resources.getString(R.string.disconnecting))
                    } else {
                        TileState.Inactive(subtitle = resources.getString(R.string.disconnecting))
                    }
                }

                is TunnelState.Error -> {
                    if (tunnelState.errorState.isBlocking) {
                        TileState.Active(subtitle = resources.getString(R.string.blocking_internet))
                    } else {
                        TileState.Inactive(subtitle = resources.getString(R.string.critical_error))
                    }
                }
            }
        } else {
            TileState.Inactive(subtitle = resources.getString(R.string.disconnected))
        }
    }

    private fun updateTileState(newState: TileState) {
        qsTile?.apply {
            state =
                when (newState) {
                    is TileState.Active -> Tile.STATE_ACTIVE
                    is TileState.Inactive -> Tile.STATE_INACTIVE
                    is TileState.Unavailable -> Tile.STATE_UNAVAILABLE
                }
            icon =
                when (newState) {
                    is TileState.Active -> securedIcon
                    is TileState.Inactive -> unsecuredIcon
                    is TileState.Unavailable -> unsecuredIcon
                }
            label = resources.getString(R.string.app_name)
            setSubtitleIfSupported(newState.subtitle)
            updateTile()
        }
    }

    private fun Tile.setSubtitleIfSupported(subtitleText: CharSequence) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            this.subtitle = subtitleText
        }
    }

    // TODO This logic should be more advanced, we should ensure user has an account setup etc.
    private fun isSetup(): Boolean {
        return applicationContext.prepareVpnSafe().isRight()
    }

    companion object {
        private val TUNNEL_STATE_DEBOUNCE = 300.milliseconds
    }
}
